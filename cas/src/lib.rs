// Copyright (c) 2022 solarliner
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash, Hasher},
    io::ErrorKind,
    marker::PhantomData,
    path::PathBuf,
};

use fxhash::FxBuildHasher;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    fs,
    io::{self, AsyncWriteExt},
};

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(code(cas::io::error))]
    Io(#[from] tokio::io::Error),
    #[error(transparent)]
    #[diagnostic(code(cas::serialization::error))]
    Serialization(#[from] bincode::Error),
}

pub struct Cas<T> {
    __value: PhantomData<T>,
    base_path: PathBuf,
    hasher: FxBuildHasher,
}

impl<T> Cas<T> {
    pub async fn new(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();
        tokio::fs::create_dir_all(&path).await?;
        Ok(Self {
            __value: PhantomData,
            base_path: path,
            hasher: FxBuildHasher::default(),
        })
    }

    async fn contains<Q: Hash>(&self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q>,
    {
        match tokio::fs::metadata(self.filename_for_key(key)).await {
            Ok(_) => Ok(true),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
            Err(err) => Err(Error::Io(err)),
        }
    }

    async fn load<Q: Hash>(&self, key: &Q) -> Result<impl io::AsyncBufRead, Error>
    where
        T: Borrow<Q>,
    {
        Ok(io::BufReader::new(
            fs::File::open(self.filename_for_key(key)).await?,
        ))
    }

    async fn save<Q: Hash>(&self, key: &Q) -> Result<impl io::AsyncWrite, Error>
    where
        T: Borrow<Q>,
    {
        Ok(io::BufWriter::new(
            fs::File::create(self.filename_for_key(key)).await?,
        ))
    }

    fn filename_for_key<Q: Hash>(&self, key: &Q) -> PathBuf
    where
        T: Borrow<Q>,
    {
        let hash = {
            let mut hash = self.hasher.build_hasher();
            key.hash(&mut hash);
            hash.finish()
        };
        let filepath = self.base_path.join(format!("{:x}", hash));
        filepath
    }
}

impl<T: for<'a> Deserialize<'a>> Cas<T> {
    pub async fn get<Q: Hash>(&self, key: &Q) -> Result<T, Error>
    where
        T: Borrow<Q>,
    {
        let data = {
            let mut v = vec![];
            let mut r = self.load(key).await?;
            tokio::io::copy_buf(&mut r, &mut v).await?;
            v
        };
        Ok(bincode::deserialize_from(&*data)?)
    }
}

impl<T: Serialize + Hash> Cas<T> {
    pub async fn set(&self, value: &T) -> Result<(), Error> {
        let mut writer = self.save(value).await?;
        let v = bincode::serialize(value)?;
        writer.write_all(&v).await?;
        Ok(())
    }
}

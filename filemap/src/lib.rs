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
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    fs,
    io::{self, AsyncBufRead, AsyncWrite},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Deserialize, Serialize)]
struct Entry<K, T> {
    key: K,
    value: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Filemap<K, T> {
    __type: PhantomData<fn(K) -> T>,
    base_path: PathBuf,
    #[serde(skip, default)]
    hasher: FxBuildHasher,
}

impl<K, T> Filemap<K, T> {
    pub async fn new(base_path: impl Into<PathBuf>) -> Result<Self> {
        let base_path = base_path.into();
        tokio::fs::create_dir_all(&base_path).await?;
        Ok(Self {
            __type: PhantomData,
            base_path,
            hasher: FxBuildHasher::default(),
        })
    }

    pub async fn insert(&self, key: K, value: T) -> Result<()>
    where
        K: Hash,
        T: Serialize,
    {
        self.insert_ref(&key, &value).await
    }

    pub async fn insert_ref(&self, key: &K, value: &T) -> Result<()>
    where
        K: Hash,
        T: Serialize,
    {
        let buf = bincode::serialize(value)?;
        let mut wrt = self.save(key).await?;
        io::copy(&mut buf.as_slice(), &mut wrt).await?;
        Ok(())
    }

    pub async fn get<Q: Hash>(&self, key: &Q) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
        K: Borrow<Q>,
    {
        match self.load(key).await {
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(Error::Io(err)),
            Ok(mut rdr) => {
                let mut buf = vec![];
                io::copy(&mut rdr, &mut buf).await?;
                bincode::deserialize(&buf)
                    .map(Some)
                    .map_err(Error::Serialization)
            }
        }
    }

    pub async fn contains<Q: Hash>(&self, key: &Q) -> Result<bool>
    where
        K: Borrow<Q>,
    {
        match fs::metadata(self.filename_for_key(key)).await {
            Ok(meta) if meta.is_file() => Ok(true),
            Ok(_) => Ok(false),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
            Err(err) => Err(Error::Io(err)),
        }
    }

    async fn load<Q: Hash>(&self, key: &Q) -> Result<impl AsyncBufRead, io::Error>
    where
        K: Borrow<Q>,
    {
        Ok(io::BufReader::new(
            fs::File::open(self.filename_for_key(key)).await?,
        ))
    }

    async fn save<Q: Hash>(&self, key: &Q) -> Result<io::BufWriter<impl AsyncWrite>>
    where
        K: Borrow<Q>,
    {
        Ok(io::BufWriter::new(
            fs::File::create(self.filename_for_key(key)).await?,
        ))
    }

    fn filename_for_key<Q: Hash>(&self, key: &Q) -> PathBuf
    where
        K: Borrow<Q>,
    {
        let hash = {
            let mut state = self.hasher.build_hasher();
            key.hash(&mut state);
            state.finish()
        };
        self.base_path.join(format!("{:x}.bin", hash))
    }
}

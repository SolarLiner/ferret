use std::{collections::BTreeMap, ops::Not, path::PathBuf, sync::Arc};

use dashmap::DashMap;
use ferret_filemap::Filemap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct TermIx(Uuid);

impl TermIx {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct DocIx(Uuid);

impl DocIx {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TermMeta {
    documents: BTreeMap<DocIx, usize>,
    display: String,
    total_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentMeta {
    terms: BTreeMap<TermIx, usize>,
    length: usize,
    num_terms: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Index {
    terms: Filemap<TermIx, TermMeta>,
    documents: Filemap<DocIx, DocumentMeta>,
    word_map: DashMap<String, TermIx>,
}

impl Index {
    pub async fn new(base_path: impl Into<PathBuf>) -> Result<Self, ferret_filemap::Error> {
        let base_path = base_path.into();
        let (terms, documents) = tokio::try_join!(
            Filemap::new(base_path.join("terms")),
            Filemap::new(base_path.join("documents"))
        )?;
        Ok(Self {
            terms,
            documents,
            word_map: DashMap::default(),
        })
    }

    pub async fn add_document(&self, text: &str) -> Result<DocIx, ferret_filemap::Error> {
        let ix = DocIx::new();

        let tokens = text
            .split_whitespace()
            .filter_map(|s| {
                let filtered = s
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>();
                filtered.is_empty().not().then_some(filtered)
            })
            .collect::<Vec<_>>();

        let meta = DocumentMeta {
            length: text.len(),
            num_terms: tokens.len(),
            terms: tokens
                .into_iter()
                .map(|s| *self.word_map.entry(s).or_insert_with(TermIx::new))
                .fold(BTreeMap::new(), |mut map, term| {
                    map.entry(term).and_modify(|n| *n += 1).or_insert(1);
                    map
                }),
        };
        self.documents.insert_ref(&ix, &meta).await?;

        Ok(ix)
    }
}

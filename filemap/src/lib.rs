// Copyright (c) 2022 solarliner
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::hash::Hash;

use ferret_cas::Cas;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Entry<K, T> {
    pub key: K,
    pub value: T,
}

impl<K: Hash, T> Hash for Entry<K, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}

impl<K: PartialEq, T> PartialEq for Entry<K, T> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }

    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        self.key.ne(&other.key)
    }
}

impl<K: Eq, T> Eq for Entry<K, T> {}

impl<K: PartialOrd, T> PartialOrd for Entry<K, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K: Ord, T> Ord for Entry<K, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

#[derive(Debug, Clone)]
pub struct Filemap<K, T> {
    storage: Cas<Entry<K, T>>,
}

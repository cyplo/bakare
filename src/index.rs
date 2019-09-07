use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::repository_item::RepositoryItem;

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize)]
pub struct IndexItem {
    relative_path: String,
    original_source_path: String,
    version: Box<[u8]>,
}

#[derive(Serialize, Deserialize)]
pub struct Index {
    items: Vec<IndexItem>,
    index_path: String,
    repository_path: String,
}

impl Index {
    pub fn new(repository_path: &Path) -> Self {
        Index {
            items: vec![],
            index_path: repository_path.join("index").to_string_lossy().to_string(),
            repository_path: repository_path.to_string_lossy().to_string(),
        }
    }

    pub fn index_file_path(&self) -> &Path {
        Path::new(&self.index_path)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> IndexIterator {
        IndexIterator {
            index: self,
            current_item_number: 0,
        }
    }

    pub fn remember(&mut self, item: RepositoryItem) {
        println!("remembering {}", item);
        self.items.push(item.into());
    }

    fn repository_item(&self, i: &IndexItem) -> RepositoryItem {
        let index_item = i.clone();
        let relative_path = Path::new(index_item.relative_path.as_str());
        let repository_path = Path::new(&self.repository_path);
        let original_source_path = Path::new(index_item.original_source_path.as_str());
        let absolute_path = repository_path.join(relative_path);
        let absolute_path = absolute_path.as_path();
        RepositoryItem::from(original_source_path, absolute_path, relative_path, index_item.version)
    }
}

impl From<RepositoryItem> for IndexItem {
    fn from(i: RepositoryItem) -> Self {
        IndexItem {
            relative_path: i.relative_path().to_string_lossy().to_string(),
            original_source_path: i.original_source_path().to_string_lossy().to_string(),
            version: i.version(),
        }
    }
}

pub struct IndexIterator<'a> {
    index: &'a Index,
    current_item_number: usize,
}

impl<'a> Iterator for IndexIterator<'a> {
    type Item = RepositoryItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index.is_empty() || self.current_item_number > self.index.len() - 1 {
            None
        } else {
            let current_item_number = self.current_item_number;
            self.current_item_number += 1;
            let index_item = &self.index.items[current_item_number];
            Some(self.index.repository_item(index_item))
        }
    }
}

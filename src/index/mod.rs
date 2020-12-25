use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::index::item::IndexItem;
use crate::repository::{item::RepositoryItem, ItemId};
use crate::version::Version;
use anyhow::Result;
use anyhow::*;

mod io;
mod item;
mod lock;

#[derive(Serialize, Deserialize)]
pub struct Index {
    newest_items_by_source_path: HashMap<String, IndexItem>,
    items_by_file_id: HashMap<ItemId, IndexItem>,
    index_path: String,
    repository_path: String,
    version: Version,
}

impl Index {
    pub fn new<T: AsRef<Path>>(repository_path: T) -> Self {
        let repository_path = repository_path.as_ref();
        Index {
            newest_items_by_source_path: Default::default(),
            items_by_file_id: Default::default(),
            index_path: repository_path.join("index").to_string_lossy().to_string(),
            repository_path: repository_path.to_string_lossy().to_string(),
            version: Version::default(),
        }
    }

    pub fn remember<S: AsRef<Path>, R: AsRef<Path>>(&mut self, original_source_path: S, relative_path: R, id: ItemId) {
        let original_source_path = original_source_path.as_ref();
        let relative_path = relative_path.as_ref();
        let item = if let Some(old) = self
            .newest_items_by_source_path
            .get(&original_source_path.to_string_lossy().to_string())
        {
            old.next_version(id, relative_path.to_string_lossy().to_string())
        } else {
            IndexItem::from(
                original_source_path.to_string_lossy().to_string(),
                relative_path.to_string_lossy().to_string(),
                id,
                Version::default(),
            )
        };

        self.items_by_file_id.insert(item.id(), item.clone());
        self.newest_items_by_source_path
            .insert(original_source_path.to_string_lossy().to_string(), item);
    }

    pub fn repository_item(&self, i: &IndexItem) -> RepositoryItem {
        let index_item = i.clone();
        let relative_path = Path::new(index_item.relative_path());
        let repository_path = Path::new(&self.repository_path);
        let original_source_path = Path::new(index_item.original_source_path());
        let absolute_path = repository_path.join(relative_path);
        let absolute_path = absolute_path.as_path();
        RepositoryItem::from(
            original_source_path,
            absolute_path,
            relative_path,
            index_item.id(),
            index_item.version(),
        )
    }

    pub fn newest_item_by_source_path<T: AsRef<Path>>(&self, path: T) -> Result<Option<IndexItem>> {
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err(anyhow!("repository path not absolute"));
        }
        Ok(self
            .newest_items_by_source_path
            .get(&path.to_string_lossy().to_string())
            .cloned())
    }

    pub fn item_by_id(&self, id: &ItemId) -> Result<Option<IndexItem>> {
        Ok(self.items_by_file_id.get(id).cloned())
    }

    pub fn newest_items(&self) -> IndexItemIterator {
        IndexItemIterator {
            iterator: self.newest_items_by_source_path.iter(),
        }
    }
}

pub struct IndexItemIterator<'a> {
    iterator: Iter<'a, String, IndexItem>,
}

impl<'a> Iterator for IndexItemIterator<'a> {
    type Item = IndexItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|i| i.1.clone())
    }
}

use std::collections::hash_map::Iter;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use vfs::VfsPath;

use crate::index::item::IndexItem;
use crate::repository::ItemId;
use crate::version::Version;
use anyhow::Result;

mod io;
pub mod item;
mod lock;

#[derive(Serialize, Deserialize, Debug)]
pub struct Index {
    newest_items_by_source_path: HashMap<String, IndexItem>,
    items_by_file_id: HashMap<ItemId, IndexItem>,
    version: Version,
}

impl Index {
    pub fn new() -> Result<Self> {
        Ok(Index {
            newest_items_by_source_path: Default::default(),
            items_by_file_id: Default::default(),
            version: Version::default(),
        })
    }

    pub fn remember(&mut self, original_source_path: &VfsPath, relative_path: &str, id: ItemId) {
        let item = if let Some(old) = self
            .newest_items_by_source_path
            .get(&original_source_path.as_str().to_string())
        {
            old.next_version(id, relative_path.to_string())
        } else {
            IndexItem::from(
                original_source_path.as_str().to_string(),
                relative_path.to_string(),
                id,
                Version::default(),
            )
        };

        self.items_by_file_id.insert(item.id(), item.clone());
        self.newest_items_by_source_path
            .insert(original_source_path.as_str().to_string(), item);
    }

    pub fn newest_item_by_source_path(&self, path: &VfsPath) -> Result<Option<IndexItem>> {
        Ok(self.newest_items_by_source_path.get(&path.as_str().to_string()).cloned())
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

#[derive(Debug)]
pub struct IndexItemIterator<'a> {
    iterator: Iter<'a, String, IndexItem>,
}

impl<'a> Iterator for IndexItemIterator<'a> {
    type Item = IndexItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|i| i.1.clone())
    }
}

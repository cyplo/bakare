use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::BakareError;
use crate::error::BakareError::RepositoryPathNotAbsolute;
use crate::repository::{ItemId, Version};
use crate::repository_item::RepositoryItem;
use std::collections::hash_map::Iter;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize)]
pub struct IndexItem {
    relative_path: String,
    original_source_path: String,
    id: ItemId,
    version: Version,
}

#[derive(Serialize, Deserialize)]
pub struct Index {
    newest_items_by_source_path: HashMap<String, IndexItem>,
    items_by_file_id: HashMap<ItemId, IndexItem>,
    index_path: String,
    repository_path: String,
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

impl IndexItem {
    fn from(original_source_path: String, relative_path: String, id: ItemId, version: Version) -> IndexItem {
        IndexItem {
            relative_path,
            original_source_path,
            id,
            version,
        }
    }

    fn next_version(&self, id: ItemId) -> IndexItem {
        IndexItem {
            relative_path: self.relative_path.clone(),
            original_source_path: self.original_source_path.clone(),
            version: self.version.next(),
            id,
        }
    }
}

impl Index {
    pub fn new(repository_path: &Path) -> Self {
        Index {
            newest_items_by_source_path: Default::default(),
            items_by_file_id: Default::default(),
            index_path: repository_path.join("index").to_string_lossy().to_string(),
            repository_path: repository_path.to_string_lossy().to_string(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, BakareError> {
        let index_file_path = path.join("index");
        let index_file = File::open(index_file_path)?;
        let index: Index = serde_cbor::from_reader(index_file)?;
        Ok(index)
    }

    pub fn save(&self) -> Result<(), BakareError> {
        let index_file = File::create(self.index_file_path())?;
        serde_cbor::to_writer(index_file, &self)?;
        Ok(())
    }

    pub fn index_file_path(&self) -> &Path {
        Path::new(&self.index_path)
    }

    pub fn len(&self) -> usize {
        self.items_by_file_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items_by_file_id.is_empty()
    }

    pub fn remember(&mut self, original_source_path: &Path, absolute_path: &Path, relative_path: &Path, id: ItemId) {
        let item = if let Some(old) = self
            .newest_items_by_source_path
            .get(&original_source_path.to_string_lossy().to_string())
        {
            old.next_version(id)
        } else {
            IndexItem::from(
                original_source_path.to_string_lossy().to_string(),
                relative_path.to_string_lossy().to_string(),
                id,
                Version::default(),
            )
        };

        self.items_by_file_id.insert(item.id.clone(), item.clone());
        self.newest_items_by_source_path
            .insert(original_source_path.to_string_lossy().to_string(), item.clone());
    }

    pub fn repository_item(&self, i: &IndexItem) -> RepositoryItem {
        let index_item = i.clone();
        let relative_path = Path::new(&index_item.relative_path);
        let repository_path = Path::new(&self.repository_path);
        let original_source_path = Path::new(&index_item.original_source_path);
        let absolute_path = repository_path.join(relative_path);
        let absolute_path = absolute_path.as_path();
        RepositoryItem::from(
            original_source_path,
            absolute_path,
            relative_path,
            index_item.id,
            index_item.version,
        )
    }

    pub fn newest_item_by_source_path(&self, path: &Path) -> Result<Option<IndexItem>, BakareError> {
        if !path.is_absolute() {
            return Err(BakareError::RepositoryPathNotAbsolute);
        }
        Ok(self
            .newest_items_by_source_path
            .get(&path.to_string_lossy().to_string())
            .cloned())
    }

    pub fn item_by_id(&self, id: &ItemId) -> Result<Option<IndexItem>, BakareError> {
        Ok(self.items_by_file_id.get(id).map(|i| i.clone()))
    }

    pub fn newest_items(&self) -> IndexItemIterator {
        IndexItemIterator {
            iterator: self.newest_items_by_source_path.iter(),
        }
    }
}

impl From<RepositoryItem> for IndexItem {
    fn from(i: RepositoryItem) -> Self {
        IndexItem {
            relative_path: i.relative_path().to_string_lossy().to_string(),
            original_source_path: i.original_source_path().to_string_lossy().to_string(),
            id: i.id().clone(),
            version: i.version().clone(),
        }
    }
}

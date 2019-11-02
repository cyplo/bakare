use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::BakareError;
use crate::repository::{ItemId, Version};
use crate::repository_item::RepositoryItem;
use glob::glob;
use std::time::Duration;
use std::{fs, thread};
use uuid::Uuid;

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

    #[serde(skip)]
    lock_id: uuid::Uuid,
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

    fn next_version(&self, id: ItemId, relative_path: String) -> IndexItem {
        IndexItem {
            original_source_path: self.original_source_path.clone(),
            version: self.version.next(),
            relative_path,
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
            lock_id: Uuid::new_v4(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, BakareError> {
        let lock_id = Uuid::new_v4();
        let index = Index::load_reusing_lock(path, lock_id)?;
        Index::release_lock(path, lock_id)?;
        Ok(index)
    }

    fn load_reusing_lock(path: &Path, lock_id: Uuid) -> Result<Self, BakareError> {
        Index::acquire_lock(lock_id, path)?;
        let index_file_path = Index::index_file_path_for_repository_path(path);
        let index_file = File::open(index_file_path)?;
        let mut index: Index = serde_cbor::from_reader(index_file)?;
        index.lock_id = lock_id;
        Ok(index)
    }

    fn index_file_path_for_repository_path(path: &Path) -> PathBuf {
        path.join("index")
    }

    pub fn save(&mut self) -> Result<(), BakareError> {
        Index::acquire_lock(self.lock_id, &self.index_directory())?;

        self.reload_and_merge()?;

        let index_file = File::create(self.index_file_path())?;
        serde_cbor::to_writer(index_file, &self)?;

        Index::release_lock(&self.index_directory(), self.lock_id)?;
        Ok(())
    }

    fn reload_and_merge(&mut self) -> Result<(), BakareError> {
        let repository_path = Path::new(&self.repository_path);
        let old_index_path = Index::index_file_path_for_repository_path(repository_path);
        if old_index_path.exists() {
            let old_index = Index::load_reusing_lock(repository_path, self.lock_id)?;
            self.items_by_file_id.extend(old_index.items_by_file_id);

            for (source_path, old_newest_item) in old_index.newest_items_by_source_path {
                if let Some(new_newest_item) = self.newest_items_by_source_path.get(&source_path) {
                    if old_newest_item.version > new_newest_item.version {
                        self.newest_items_by_source_path.insert(source_path, old_newest_item);
                    }
                } else {
                    self.newest_items_by_source_path.insert(source_path, old_newest_item);
                }
            }
        }
        Ok(())
    }

    fn release_lock(path: &Path, lock_id: Uuid) -> Result<(), BakareError> {
        let lock_file_path = Index::lock_file_path(path, lock_id);
        fs::remove_file(lock_file_path)?;
        Ok(())
    }

    fn lock_file_path(path: &Path, lock_id: Uuid) -> String {
        format!("{}/{}.lock", path.to_string_lossy(), lock_id)
    }

    fn acquire_lock(lock_id: Uuid, index_directory: &Path) -> Result<(), BakareError> {
        let lock_file_path = Index::lock_file_path(index_directory, lock_id);
        Index::wait_for_only_my_locks_left(lock_id, index_directory)?;
        File::create(lock_file_path)?;
        Ok(())
    }

    fn wait_for_only_my_locks_left(lock_id: Uuid, index_directory: &Path) -> Result<(), BakareError> {
        let parent_directory_path = index_directory.to_string_lossy();
        let lock_file_extension = "lock";
        let my_lock_file_path = format!("{}/{}.{}", parent_directory_path, lock_id, lock_file_extension);

        loop {
            let mut locks = glob(&format!("{}/*.{}", parent_directory_path, lock_file_extension))?;
            {
                let only_my_locks = locks.all(|path| match path {
                    Ok(path) => path.to_string_lossy() == my_lock_file_path,
                    Err(_) => false,
                });
                if only_my_locks {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    fn index_directory(&self) -> PathBuf {
        self.index_file_path().parent().unwrap().to_path_buf()
    }

    pub fn index_file_path(&self) -> PathBuf {
        Path::new(&self.index_path).to_path_buf()
    }

    pub fn remember(&mut self, original_source_path: &Path, relative_path: &Path, id: ItemId) {
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
        Ok(self.items_by_file_id.get(id).cloned())
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

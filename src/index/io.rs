use crate::error::BakareError;
use crate::index::item::IndexItem;
use crate::index::{lock, Index};
use crate::repository::ItemId;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use uuid::Uuid;

impl Index {
    pub fn load(path: &Path) -> Result<Self, BakareError> {
        let lock_id = Uuid::new_v4();
        let index = Index::load_reusing_lock(path, lock_id)?;
        lock::release_lock(path, lock_id)?;
        Ok(index)
    }

    pub fn save(&mut self) -> Result<(), BakareError> {
        lock::acquire_lock(self.lock_id, &self.index_directory())?;

        self.reload_and_merge()?;

        let index_file =
            File::create(self.index_file_path()).map_err(|e| (e, self.index_file_path().to_string_lossy().to_string()))?;
        serde_cbor::to_writer(index_file, &self)?;

        lock::release_lock(&self.index_directory(), self.lock_id)?;
        Ok(())
    }

    fn index_file_path(&self) -> PathBuf {
        Path::new(&self.index_path).to_path_buf()
    }

    fn load_reusing_lock(path: &Path, lock_id: Uuid) -> Result<Self, BakareError> {
        lock::acquire_lock(lock_id, path)?;
        let index_file_path = Index::index_file_path_for_repository_path(path);
        let index_file = File::open(index_file_path.clone()).map_err(|e| (e, index_file_path.to_string_lossy().to_string()))?;
        let mut index: Index = serde_cbor::from_reader(index_file)?;
        index.lock_id = lock_id;
        Ok(index)
    }

    fn reload_and_merge(&mut self) -> Result<(), BakareError> {
        let repository_path = Path::new(&self.repository_path);
        let old_index_path = Index::index_file_path_for_repository_path(repository_path);
        if old_index_path.exists() {
            let old_index = Index::load_reusing_lock(repository_path, self.lock_id)?;
            self.merge_items_by_file_id(old_index.items_by_file_id);
            self.merge_newest_items(old_index.newest_items_by_source_path);
        }
        Ok(())
    }

    fn merge_newest_items(&mut self, old_newest_items: HashMap<String, IndexItem>) {
        for (source_path, old_newest_item) in old_newest_items {
            if let Some(new_newest_item) = self.newest_items_by_source_path.get(&source_path) {
                if old_newest_item.version() > new_newest_item.version() {
                    self.newest_items_by_source_path.insert(source_path, old_newest_item);
                }
            } else {
                self.newest_items_by_source_path.insert(source_path, old_newest_item);
            }
        }
    }

    fn merge_items_by_file_id(&mut self, old_items_by_file_id: HashMap<ItemId, IndexItem>) {
        self.items_by_file_id.extend(old_items_by_file_id);
    }

    fn index_file_path_for_repository_path(path: &Path) -> PathBuf {
        path.join("index")
    }

    fn index_directory(&self) -> PathBuf {
        self.index_file_path().parent().unwrap().to_path_buf()
    }
}

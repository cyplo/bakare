use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use atomicwrites::AtomicFile;
use atomicwrites::*;
use uuid::Uuid;

use glob::glob;
use glob::Paths;

use crate::error::BakareError;
use crate::index::item::IndexItem;
use crate::index::{lock, Index};
use crate::repository::ItemId;

impl Index {
    pub fn load(path: &Path) -> Result<Self, BakareError> {
        let lock_id = Uuid::new_v4();
        lock::acquire_lock(lock_id, path)?;
        let mut index = Index::load_reusing_lock(&Index::index_file_path_for_repository_path(path), lock_id)?;
        index.absorb_other_no_lock()?;
        lock::release_lock(path, lock_id)?;
        Ok(index)
    }

    pub fn save(&mut self) -> Result<(), BakareError> {
        lock::acquire_lock(self.lock_id, &self.index_directory())?;

        let sole_lock = lock::sole_lock(self.lock_id, &self.index_directory())?;
        if sole_lock {
            self.write_index_to_file(self.index_file_path())?;
        } else {
            self.write_index_to_file(self.side_index_file_path())?;
        }

        lock::release_lock(&self.index_directory(), self.lock_id)?;
        Ok(())
    }

    pub fn absorb_other(&mut self) -> Result<(), BakareError> {
        lock::acquire_lock(self.lock_id, &self.index_directory())?;
        self.absorb_other_no_lock()?;
        lock::release_lock(&self.index_directory(), self.lock_id)?;
        Ok(())
    }

    fn absorb_other_no_lock(&mut self) -> Result<(), BakareError> {
        let sole_lock = lock::sole_lock(self.lock_id, &self.index_directory())?;
        if sole_lock {
            self.write_index_to_file(self.side_index_file_path())?;

            let indexes = self.all_side_indexes()?;
            for index in indexes {
                self.merge_with(&index?)?;
            }
        }

        Ok(())
    }

    fn write_index_to_file<T>(&mut self, path: T) -> Result<(), BakareError>
    where
        T: AsRef<Path>,
    {
        fs::create_dir_all(path.as_ref().parent().unwrap()).map_err(|e| (e, &path))?;

        let file = AtomicFile::new(&path, AllowOverwrite);
        file.write(|f| serde_cbor::to_writer(f, &self)).map_err(|e| match e {
            atomicwrites::Error::Internal(e) => BakareError::from((e, &path)),
            atomicwrites::Error::User(e) => BakareError::from(e),
        })?;

        Ok(())
    }

    fn index_file_path(&self) -> PathBuf {
        Path::new(&self.index_path).to_path_buf()
    }

    fn side_index_file_path(&self) -> PathBuf {
        self.side_indexes_path().join(format!("{}", self.lock_id))
    }

    fn side_indexes_path(&self) -> PathBuf {
        Path::new(&self.repository_path).join("side_indexes")
    }

    fn all_side_indexes(&self) -> Result<Paths, BakareError> {
        let glob_pattern = format!("{}/*", self.side_indexes_path().to_string_lossy());
        Ok(glob(&glob_pattern)?)
    }

    fn load_reusing_lock(index_file_path: &Path, lock_id: Uuid) -> Result<Self, BakareError> {
        let index_file = File::open(&index_file_path).map_err(|e| (e, index_file_path))?;
        let mut index: Index = serde_cbor::from_reader(index_file)?;
        index.lock_id = lock_id;
        index.index_path = index_file_path.to_string_lossy().to_string();
        Ok(index)
    }

    fn merge_with(&mut self, other_index_path: &PathBuf) -> Result<(), BakareError> {
        let old_index = Index::load_reusing_lock(other_index_path, self.lock_id)?;
        {
            self.merge_items_by_file_id(old_index.items_by_file_id);
            self.merge_newest_items(old_index.newest_items_by_source_path);
        }
        fs::remove_file(&other_index_path).map_err(|e| (e, &self.index_path))?;
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

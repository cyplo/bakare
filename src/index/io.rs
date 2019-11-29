use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use async_log::span;
use glob::glob;
use glob::Paths;
use uuid::Uuid;

use crate::index::item::IndexItem;
use crate::index::{lock, Index};
use crate::repository::ItemId;
use anyhow::Context;
use anyhow::Result;
use async_log::*;
use std::io::Write;

impl Index {
    pub fn load(path: &Path) -> Result<Self> {
        span!("loading index from {}", path.to_string_lossy(), {
            let lock_id = Uuid::new_v4();
            let mut index = Index::load_reusing_lock(&Index::index_file_path_for_repository_path(path), lock_id)?;
            lock::acquire_lock(lock_id, path)?;
            index.absorb_other_no_lock()?;
            lock::release_lock(path, lock_id)?;
            Ok(index)
        })
    }

    pub fn save(&mut self) -> Result<()> {
        span!("saving index with lock id {}", self.lock_id, {
            lock::acquire_lock(self.lock_id, &self.index_directory())?;

            let sole_lock = lock::sole_lock(self.lock_id, &self.index_directory())?;
            if sole_lock {
                span!("saving to {} ", self.index_file_path().to_string_lossy(), {
                    self.write_index_to_file(self.index_file_path())?;
                });
            } else {
                span!("saving to {} ", self.side_index_file_path().to_string_lossy(), {
                    self.write_index_to_file(self.side_index_file_path())?;
                });
            }

            lock::release_lock(&self.index_directory(), self.lock_id)?;
            Ok(())
        })
    }

    pub fn absorb_other(&mut self) -> Result<()> {
        lock::acquire_lock(self.lock_id, &self.index_directory())?;
        self.absorb_other_no_lock()?;
        lock::release_lock(&self.index_directory(), self.lock_id)?;
        Ok(())
    }

    fn absorb_other_no_lock(&mut self) -> Result<()> {
        let indexes = self.all_side_indexes()?;
        for index in indexes {
            self.merge_with(&index?)?;
        }
        let sole_lock = lock::sole_lock(self.lock_id, &self.index_directory())?;
        if sole_lock {
            self.write_index_to_file(self.side_index_file_path())?;
        }

        Ok(())
    }

    fn write_index_to_file<T>(&mut self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        span!("write index to file: {}", path.as_ref().to_string_lossy(), {
            fs::create_dir_all(path.as_ref().parent().unwrap()).context("create index directory")?;

            let mut file = File::create(path.as_ref()).context("create index file")?;
            let contents = serde_json::to_string(&self).context("index serialization")?;
            file.write_all(contents.as_bytes()).context("writing index to disk")?;
            file.sync_all().context("syncing")?;
        });
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

    fn all_side_indexes(&self) -> Result<Paths> {
        let glob_pattern = format!("{}/*", self.side_indexes_path().to_string_lossy());
        Ok(glob(&glob_pattern)?)
    }

    fn load_reusing_lock<T: AsRef<Path>>(index_file_path: T, lock_id: Uuid) -> Result<Self> {
        let path_text = format!("{}", index_file_path.as_ref().to_string_lossy());
        span!("load {} reusing lock {}", path_text, lock_id, {
            let index_text = fs::read_to_string(path_text.clone()).context("reading index file contents")?;

            let mut index: Index =
                serde_json::from_str(&index_text).context(format!("cannot read index from: {}", index_text))?;
            index.lock_id = lock_id;
            index.index_path = path_text.clone();
            Ok(index)
        })
    }

    fn merge_with(&mut self, other_index_path: &PathBuf) -> Result<()> {
        span!("merging {} into {}", other_index_path.to_string_lossy(), self.index_path, {
            let old_index = Index::load_reusing_lock(other_index_path, self.lock_id)?;
            {
                self.merge_items_by_file_id(old_index.items_by_file_id);
                self.merge_newest_items(old_index.newest_items_by_source_path);
            }
            fs::remove_file(&other_index_path)?;
        });
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

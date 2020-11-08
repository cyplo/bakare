use atomicwrites::{AllowOverwrite, AtomicFile};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::index::item::IndexItem;
use crate::index::{lock, Index};
use crate::repository::ItemId;
use anyhow::Result;
use anyhow::*;
use lock::Lock;
use nix::unistd::getpid;
use std::{cmp::max, io::Write};

impl Index {
    pub fn load(repository_path: &Path) -> Result<Self> {
        if !repository_path.exists() {
            let mut index = Index::new(repository_path);
            index.save()?;
        }
        let lock = Lock::new(repository_path)?;
        let index = Index::load_from_file(&Index::index_file_path_for_repository_path(repository_path))?;
        lock.release()?;
        log::debug!(
            "[{}] loaded index from {}, version: {}",
            getpid(),
            repository_path.to_string_lossy(),
            index.version
        );
        Ok(index)
    }

    pub fn save(&mut self) -> Result<()> {
        let lock_id = Uuid::new_v4();
        let lock = Lock::new(&self.index_directory()?)?;
        if self.index_file_path().exists() {
            let index = Index::load_from_file(&Index::index_file_path_for_repository_path(&self.index_directory()?))?;
            self.merge_items_by_file_id(index.items_by_file_id);
            self.merge_newest_items(index.newest_items_by_source_path);
            self.version = max(self.version.clone(), index.version);
        }
        self.version = self.version.next();
        self.write_index_to_file(self.index_file_path())?;
        lock.release()?;
        log::debug!("[{}] saved index version {} with lock id {}", getpid(), self.version, lock_id,);
        Ok(())
    }

    fn write_index_to_file<T>(&mut self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        fs::create_dir_all(
            path.as_ref()
                .parent()
                .ok_or_else(|| anyhow!("cannot compute parent path for {}", path.as_ref().to_string_lossy()))?,
        )
        .context("create index directory")?;

        let file = AtomicFile::new(&path, AllowOverwrite);

        file.write(|f| {
            let contents = serde_json::to_string(&self)?;
            f.write_all(contents.as_bytes())
        })
        .context("writing index to disk")?;

        Ok(())
    }

    fn index_file_path(&self) -> PathBuf {
        Path::new(&self.index_path).to_path_buf()
    }

    fn load_from_file<T: AsRef<Path>>(index_file_path: T) -> Result<Self> {
        let path_text = format!("{}", index_file_path.as_ref().to_string_lossy());
        let index_text =
            fs::read_to_string(path_text.clone()).context(format!("reading index file contents from {}", path_text))?;

        let mut index: Index = serde_json::from_str(&index_text).context(format!("cannot read index from: {}", index_text))?;
        index.index_path = path_text;
        Ok(index)
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

    fn index_directory(&self) -> Result<PathBuf> {
        Ok(self
            .index_file_path()
            .parent()
            .ok_or_else(|| anyhow!("cannot compute parent path for {}", self.index_file_path().to_string_lossy()))?
            .to_path_buf())
    }
}

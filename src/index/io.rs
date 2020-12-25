use std::collections::HashMap;
use vfs::VfsPath;

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
    pub fn load(repository_path: &VfsPath) -> Result<Self> {
        if !repository_path.exists() {
            let mut index = Index::new()?;
            index.save(repository_path)?;
        }
        let lock = Lock::lock(repository_path)?;
        let index_file_path = &Index::index_file_path_for_repository_path(repository_path)?;
        let index = Index::load_from_file(index_file_path)?;
        lock.release()?;
        log::debug!(
            "[{}] loaded index from {}, version: {}; {} items",
            getpid(),
            index_file_path.as_str(),
            index.version,
            index.newest_items_by_source_path.len()
        );
        Ok(index)
    }

    pub fn save(&mut self, repository_path: &VfsPath) -> Result<()> {
        let lock_id = Uuid::new_v4();
        let lock = Lock::lock(repository_path)?;

        let index_file_path = &Index::index_file_path_for_repository_path(repository_path)?;
        if index_file_path.exists() {
            let index = Index::load_from_file(&Index::index_file_path_for_repository_path(repository_path)?)?;
            self.merge_items_by_file_id(index.items_by_file_id);
            self.merge_newest_items(index.newest_items_by_source_path);
            self.version = max(self.version, index.version);
        }
        self.version = self.version.next();
        self.write_index_to_file(index_file_path)?;
        lock.release()?;
        log::debug!(
            "[{}] saved index version {} with lock id {} to {}; {} items",
            getpid(),
            self.version,
            lock_id,
            index_file_path.as_str(),
            self.newest_items_by_source_path.len()
        );
        Ok(())
    }

    fn write_index_to_file(&mut self, index_file_path: &VfsPath) -> Result<()> {
        let parent = index_file_path.parent();
        match parent {
            None => Err(anyhow!(format!("cannot get parent for {}", index_file_path.as_str()))),
            Some(parent) => Ok(parent
                .create_dir_all()
                .context(format!("create index directory at {}", index_file_path.as_str()))?),
        }?;

        let contents;
        {
            let mut file = index_file_path.create_file()?;
            contents = serde_json::to_string(&self)?;
            file.write_all(contents.as_bytes()).context("writing index to disk")?;
            file.flush()?;
        }
        let readback = index_file_path.read_to_string()?;
        if readback != contents {
            Err(anyhow!("index readback incorrect"))
        } else {
            Ok(())
        }
    }

    fn load_from_file(index_file_path: &VfsPath) -> Result<Self> {
        let index_text = index_file_path
            .read_to_string()
            .context(format!("reading index file contents from {}", index_file_path.as_str()))?;

        let index: Index = serde_json::from_str(&index_text).context(format!("cannot read index from: {}", index_text))?;
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

    fn index_file_path_for_repository_path(path: &VfsPath) -> Result<VfsPath> {
        Ok(path.join("index")?)
    }
}

#[cfg(test)]
mod must {
    use crate::index::Index;
    use anyhow::Result;
    use vfs::{MemoryFS, VfsPath};

    #[test]
    fn have_version_increased_when_saved() -> Result<()> {
        let temp_dir: VfsPath = MemoryFS::new().into();
        let mut index = Index::new()?;
        let old_version = index.version;

        index.save(&temp_dir)?;

        let new_version = index.version;

        assert!(new_version > old_version);

        Ok(())
    }
}

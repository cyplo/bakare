use std::fs;
use std::path::Path;

use crate::error::BakareError;
use crate::IndexVersion;
use crate::IndexViewReadonly;
use crate::ItemVersion;

/// represents a place where backup is stored an can be restored from.
/// right now only on-disk directory storage is supported
/// repository always knows the newest version of the index and is responsible for syncing the index to disk
/// and making sure that different threads can access index in parallel
pub struct Repository<'a> {
    /// absolute path to where the repository is stored on disk
    path: &'a Path,
    index: IndexViewReadonly<'a>,
    newest_index_version: IndexVersion,
}

#[derive(Copy, Clone)]
pub struct RepositoryItem<'a> {
    version: ItemVersion<'a>,
}

pub struct RepositoryIterator<'a> {
    version: IndexVersion,
    index: &'a IndexViewReadonly<'a>,
    current_item_number: usize,
}

impl<'a> Iterator for RepositoryIterator<'a> {
    type Item = RepositoryItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_item_number >= self.index.items.len() - 1 {
            None
        } else {
            let current_item_number = self.current_item_number;
            self.current_item_number += 1;
            Some(self.index.items[current_item_number])
        }
    }
}

impl<'a> RepositoryItem<'a> {
    pub fn version(&self) -> &ItemVersion {
        &self.version
    }
}

impl<'a> Repository<'a> {
    pub fn open(path: &Path) -> Result<Repository, BakareError> {
        // TODO open index from file

        let version = IndexVersion;
        Ok(Repository {
            path,
            index: IndexViewReadonly {
                index_version: version,
                items: vec![],
            },
            newest_index_version: version,
        })
    }

    pub fn iter(&self) -> RepositoryIterator {
        RepositoryIterator {
            index: &self.index,
            version: self.index.index_version,
            current_item_number: 0,
        }
    }

    pub fn store(&self, source_path: &Path) -> Result<(), BakareError> {
        // get file id -> contents hash + original path + time of taking notes
        // get storage path for File
        // store file contents
        // remember File

        let destination_path = self.path.join(source_path);
        if source_path.is_dir() {
            fs::create_dir(destination_path.clone())?;
        }
        if source_path.is_file() {
            fs::copy(source_path, destination_path.clone())?;
        }

        // TODO create new version, remember source_path

        Ok(())
    }

    pub fn item(&self, path: &Path) -> Option<RepositoryItem> {
        unimplemented!()
    }

    pub fn newest_version_for(&self, source_path: &Path) -> Result<ItemVersion, BakareError> {
        unimplemented!()
    }
}

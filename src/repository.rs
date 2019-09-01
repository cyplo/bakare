use std::fs;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::error::BakareError;
use crate::IndexVersion;
use crate::IndexViewReadonly;
use crate::ItemVersion;
use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

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

#[derive(Clone)]
pub struct RepositoryItem<'a> {
    version: ItemVersion<'a>,
    relative_path: Rc<Path>,
}

pub struct RepositoryIterator<'a> {
    version: IndexVersion,
    index: &'a IndexViewReadonly<'a>,
    current_item_number: usize,
}

impl<'a> Iterator for RepositoryIterator<'a> {
    type Item = &'a RepositoryItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index.items.is_empty() || self.current_item_number >= self.index.items.len() - 1 {
            None
        } else {
            let current_item_number = self.current_item_number;
            self.current_item_number += 1;
            Some(&self.index.items[current_item_number])
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
        let walker = WalkDir::new(path);
        let all_files: Result<Vec<DirEntry>, _> = walker
            .into_iter()
            .filter_entry(|e| e.path() != path && !e.path().is_dir())
            .collect();
        let all_files = all_files?;
        let all_items: Vec<RepositoryItem> = all_files
            .into_iter()
            .map(|p| RepositoryItem {
                version: ItemVersion(""),
                relative_path: Rc::from(p.path()),
            })
            .collect();

        let version = IndexVersion;
        Ok(Repository {
            path,
            index: IndexViewReadonly {
                index_version: version,
                items: all_items,
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
        if source_path.is_file() {}

        // TODO create new version, remember source_path

        Ok(())
    }

    pub fn item(&self, path: &Path) -> Option<&RepositoryItem> {
        None
    }

    pub fn newest_version_for(&self, item: RepositoryItem) -> ItemVersion {
        ItemVersion("")
    }
}

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

#[derive(Clone, Debug)]
pub struct RepositoryItem<'a> {
    version: ItemVersion<'a>,
    relative_path: Box<Path>,
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
    fn get_all_files_recursively(path: &Path) -> Result<Vec<Box<Path>>, BakareError> {
        let path_text = path.to_string_lossy();
        let walker = WalkDir::new(path);

        let mut result = vec![];

        for maybe_entry in walker {
            let entry = maybe_entry?;
            if entry.path() != path {
                if entry.path().is_file() {
                    result.push(Box::from(path));
                }
                if entry.path().is_dir() {
                    let children = Repository::get_all_files_recursively(entry.path())?;

                    for child in children {
                        result.push(child);
                    }
                }
            }
        }

        Ok(result)
    }
    pub fn open(path: &Path) -> Result<Repository, BakareError> {
        if !path.is_absolute() {
            return Err(BakareError::RepositoryPathNotAbsolute);
        }
        println!("opening repository at {}", path.display());

        let all_files = Repository::get_all_files_recursively(path)?;
        let all_items: Vec<RepositoryItem> = all_files
            .into_iter()
            .map(|p| RepositoryItem {
                version: ItemVersion(""),
                relative_path: p,
            })
            .collect();

        let version = IndexVersion;
        println!("opened repository at {} - {} items present", path.display(), all_items.len());
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

    pub fn store(&mut self, source_path: &Path) -> Result<(), BakareError> {
        if !source_path.is_absolute() {
            return Err(BakareError::PathToStoreNotAbsolute);
        }
        let destination_path: &str = &(self.path.to_string_lossy() + source_path.to_string_lossy());
        let destination_path = Path::new(&destination_path);
        if source_path == destination_path {
            return Err(BakareError::SourceSameAsRepository);
        }
        if source_path.is_dir() {
            fs::create_dir(destination_path)?;
        }
        if source_path.is_file() {
            println!("storing {} as {}", source_path.display(), destination_path.display());
            fs::create_dir_all(destination_path.parent().unwrap())?;
            fs::copy(source_path, destination_path)?;
            self.index.items.push(RepositoryItem {
                version: ItemVersion(""),
                relative_path: Box::from(destination_path),
            });
        }
        Ok(())
    }

    pub fn item(&self, path: &Path) -> Option<&RepositoryItem> {
        self.index.items.iter().find(|i| *i.relative_path == *path)
    }

    pub fn newest_version_for(&self, item: RepositoryItem) -> ItemVersion {
        ItemVersion("")
    }
}

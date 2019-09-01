use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::error::BakareError;
use crate::index::{Index, IndexIterator};
use crate::repository_item::RepositoryItem;
use serde::{Deserialize, Serialize};
use std::fs::File;

/// represents a place where backup is stored an can be restored from.
/// right now only on-disk directory storage is supported
/// repository always knows the newest version of the index and is responsible for syncing the index to disk
/// and making sure that different threads can access index in parallel
pub struct Repository<'a> {
    /// absolute path to where the repository is stored on disk
    path: &'a Path,
    index: Index,
}

pub struct RepositoryIterator<'a> {
    index_iterator: IndexIterator<'a>,
}

impl<'a> Iterator for RepositoryIterator<'a> {
    type Item = RepositoryItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.index_iterator.next()
    }
}

impl<'a> Repository<'a> {
    pub fn init(path: &Path) -> Result<(), BakareError> {
        let index = Index::new(path);
        let index_file = File::create(index.index_file_path())?;
        serde_cbor::to_writer(index_file, &index)?;
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Repository, BakareError> {
        if !path.is_absolute() {
            return Err(BakareError::RepositoryPathNotAbsolute);
        }

        let index_file_path = path.join("index");
        let index_file = File::open(index_file_path)?;
        let index: Index = serde_cbor::from_reader(index_file)?;

        Ok(Repository { path, index })
    }

    pub fn iter(&self) -> RepositoryIterator {
        RepositoryIterator {
            index_iterator: self.index.iter(),
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

            self.index.remember(RepositoryItem::from(
                source_path,
                destination_path,
                destination_path.strip_prefix(self.path)?,
            ));
        }
        Ok(())
    }

    pub fn item_by_source_path(&self, path: &Path) -> Result<Option<RepositoryItem>, BakareError> {
        println!(
            "trying to find {} in a repo [{}] of {} items",
            path.display(),
            self.path.display(),
            self.index.len()
        );
        if !path.is_absolute() {
            return Err(BakareError::RepositoryPathNotAbsolute);
        }
        Ok(self.iter().find(|i| i.original_source_path() == path))
    }

    fn get_all_files_recursively(path: &Path) -> Result<Vec<Box<Path>>, BakareError> {
        let walker = WalkDir::new(path);

        let mut result = vec![];

        for maybe_entry in walker {
            let entry = maybe_entry?;
            if entry.path() == path {
                continue;
            }
            if entry.path().is_file() {
                result.push(Box::from(entry.path()));
            }
        }

        Ok(result)
    }
}

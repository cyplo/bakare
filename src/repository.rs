use std::fs;
use std::path::Path;

use crate::error::BakareError;
use crate::index::{Index, IndexIterator};
use crate::repository_item::RepositoryItem;
use sha2::Digest;
use sha2::Sha512;
use std::fs::File;
use std::io::{BufReader, Read};

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
            let version = Repository::calculate_initial_version(source_path)?;
            fs::copy(source_path, destination_path)?;

            self.index.remember(RepositoryItem::from(
                source_path,
                destination_path,
                destination_path.strip_prefix(self.path)?,
                Box::from(version),
            ));
        }
        Ok(())
    }

    fn calculate_initial_version(source_path: &Path) -> Result<Box<[u8]>, BakareError> {
        let source_file = File::open(source_path)?;
        let mut reader = BufReader::new(source_file);
        let mut buffer = Vec::with_capacity(1024);
        let mut hasher = Sha512::new();

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.input(&buffer);
        }

        let version = hasher.result();
        Ok(Box::from(version.as_slice()))
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
}

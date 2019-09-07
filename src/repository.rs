use std::path::Path;
use std::{fmt, fs, io};

use crate::error::BakareError;
use crate::index::{Index, IndexIterator};
use crate::repository_item::RepositoryItem;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha512;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;

/// represents a place where backup is stored an can be restored from.
/// right now only on-disk directory storage is supported
/// repository always knows the newest version of the index and is responsible for syncing the index to disk
/// and making sure that different threads can access index in parallel
pub struct Repository<'a> {
    /// absolute path to where the repository is stored on disk
    path: &'a Path,
    index: Index,
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize)]
pub struct ItemVersion(Box<[u8]>);

impl AsRef<[u8]> for ItemVersion {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for ItemVersion {
    fn from(a: &[u8]) -> Self {
        ItemVersion(Box::from(a))
    }
}

impl fmt::Display for ItemVersion {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self))
    }
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
        index.save()?;
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Repository, BakareError> {
        if !path.is_absolute() {
            return Err(BakareError::RepositoryPathNotAbsolute);
        }

        let index = Index::load(path)?;
        Ok(Repository { path, index })
    }

    pub fn iter(&self) -> RepositoryIterator {
        RepositoryIterator {
            index_iterator: self.index.iter(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn store(&mut self, source_path: &Path) -> Result<(), BakareError> {
        if !source_path.is_absolute() {
            return Err(BakareError::PathToStoreNotAbsolute);
        }
        let version = Repository::calculate_version(source_path)?;
        let destination_path = self.path.join(version.to_string());
        let destination_path = Path::new(&destination_path);

        if source_path.is_file() {
            fs::create_dir_all(destination_path.parent().unwrap())?;
            fs::copy(source_path, destination_path)?;

            self.index.remember(RepositoryItem::from(
                source_path,
                destination_path,
                destination_path.strip_prefix(self.path)?,
                version,
            ));
            self.index.save()?;
        }
        Ok(())
    }

    fn calculate_version(source_path: &Path) -> Result<ItemVersion, BakareError> {
        let source_file = File::open(source_path)?;
        let mut reader = BufReader::new(source_file);
        let mut hasher = Sha512::new();

        io::copy(&mut reader, &mut hasher)?;

        Ok(hasher.result().as_slice().into())
    }

    pub fn item_by_source_path_and_version(
        &self,
        path: &Path,
        version: &ItemVersion,
    ) -> Result<Option<RepositoryItem>, BakareError> {
        if !path.is_absolute() {
            return Err(BakareError::RepositoryPathNotAbsolute);
        }

        Ok(self
            .iter()
            .find(|i| i.original_source_path() == path && i.version() == version))
    }

    pub fn item_by_source_path(&self, path: &Path) -> Result<Option<RepositoryItem>, BakareError> {
        if !path.is_absolute() {
            return Err(BakareError::RepositoryPathNotAbsolute);
        }
        Ok(self.iter().find(|i| i.original_source_path() == path))
    }
}

use std::path::Path;
use std::{fmt, fs, io};

use crate::error::BakareError;
use crate::index::{Index, IndexItem, IndexItemIterator};
use crate::repository_item::RepositoryItem;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha512;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;
use std::ops::Add;

/// represents a place where backup is stored an can be restored from.
/// right now only on-disk directory storage is supported
/// repository always knows the newest version of the index and is responsible for syncing the index to disk
/// and making sure that different threads can access index in parallel
pub struct Repository<'a> {
    /// absolute path to where the repository is stored on disk
    path: &'a Path,
    index: Index,
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize, Hash)]
pub struct ItemId(Box<[u8]>);

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize, Hash)]
pub struct Version(u128);

pub struct RepositoryItemIterator<'a> {
    iterator: IndexItemIterator<'a>,
    index: &'a Index,
}

impl<'a> Iterator for RepositoryItemIterator<'a> {
    type Item = RepositoryItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|i| self.index.repository_item(&i))
    }
}

impl Version {
    pub fn next(&self) -> Self {
        Version(self.0 + 1)
    }
}

impl Default for Version {
    fn default() -> Self {
        Version(1)
    }
}

impl AsRef<[u8]> for ItemId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for ItemId {
    fn from(a: &[u8]) -> Self {
        ItemId(Box::from(a))
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self))
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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn store(&mut self, source_path: &Path) -> Result<(), BakareError> {
        if !source_path.is_absolute() {
            return Err(BakareError::PathToStoreNotAbsolute);
        }
        let id = Repository::calculate_id(source_path)?;
        let destination_path = self.path.join(id.to_string());
        let destination_path = Path::new(&destination_path);

        if source_path.is_file() {
            fs::create_dir_all(destination_path.parent().unwrap())?;
            fs::copy(source_path, destination_path)?;

            self.index
                .remember(source_path, destination_path, destination_path.strip_prefix(self.path)?, id);
            self.index.save()?;
        }
        Ok(())
    }

    pub fn newest_item_by_source__path(&self, path: &Path) -> Result<Option<RepositoryItem>, BakareError> {
        Ok(self
            .index
            .newest_item_by_source_path(path)?
            .map(|i| self.index.repository_item(&i)))
    }

    pub fn item_by_id(&self, id: &ItemId) -> Result<Option<RepositoryItem>, BakareError> {
        self.index.item_by_id(id).map(|i| i.map(|i| self.index.repository_item(&i)))
    }

    pub fn newest_items(&self) -> RepositoryItemIterator {
        RepositoryItemIterator {
            iterator: self.index.newest_items(),
            index: &self.index,
        }
    }

    fn calculate_id(source_path: &Path) -> Result<ItemId, BakareError> {
        let source_file = File::open(source_path)?;
        let mut reader = BufReader::new(source_file);
        let mut hasher = Sha512::new();

        io::copy(&mut reader, &mut hasher)?;

        Ok(hasher.result().as_slice().into())
    }
}

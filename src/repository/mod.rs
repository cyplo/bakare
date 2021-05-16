pub mod item;

use std::fmt::{Debug, Formatter};
use std::io::BufReader;
use std::path::Path;
use std::{fmt, io};

use crate::index::{Index, IndexItemIterator};
use anyhow::Result;
use anyhow::*;
use item::RepositoryItem;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha512;
use vfs::{VfsFileType, VfsPath};

/// represents a place where backup is stored an can be restored from.
/// right now only on-disk directory storage is supported
/// repository always knows the newest version of the index and is responsible for syncing the index to disk
/// and making sure that different threads can access index in parallel
#[derive(Debug)]
pub struct Repository {
    /// path to where the repository is stored on disk
    path: VfsPath,
    index: Index,
}

const DATA_DIR_NAME: &str = "data";

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize, Hash)]
pub struct ItemId(#[serde(with = "base64")] Vec<u8>);

#[derive(Debug)]
pub struct RepositoryItemIterator<'a> {
    repository: &'a Repository,
    iterator: IndexItemIterator<'a>,
}

//TODO: move to serializers::base64
mod base64 {
    use ::base64;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        base64::decode(s).map_err(de::Error::custom)
    }
}

impl<'a> Iterator for RepositoryItemIterator<'a> {
    type Item = RepositoryItem;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iterator.next();
        match item {
            None => None,
            Some(item) => self.repository.repository_item(&item).ok(),
        }
    }
}

impl AsRef<[u8]> for ItemId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for ItemId {
    fn from(a: &[u8]) -> Self {
        ItemId(a.into())
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self))
    }
}

impl Debug for ItemId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}

impl<'a> Repository {
    pub fn init(path: &VfsPath) -> Result<Repository> {
        path.create_dir_all()?;
        let mut index = Index::new()?;
        index.save(path)?;
        let repository = Repository::open(path)?;
        repository.data_dir()?.create_dir_all()?;
        Ok(repository)
    }

    pub fn open(path: &VfsPath) -> Result<Repository> {
        let index = Index::load(path)?;
        let repository = Repository {
            path: path.clone(),
            index,
        };

        Ok(repository)
    }

    pub fn path(&self) -> &VfsPath {
        &self.path
    }

    pub fn save_index(&mut self) -> Result<()> {
        self.index.save(&self.path)
    }

    pub fn store(&mut self, source_path: &VfsPath) -> Result<()> {
        let id = Repository::calculate_id(source_path)?;
        let destination = self.data_dir()?;
        let destination = destination.join(&id.to_string())?;

        if source_path.metadata()?.file_type != VfsFileType::File {
            return Ok(());
        }
        let parent = destination
            .parent()
            .ok_or_else(|| anyhow!("cannot compute parent path for {}", &destination.as_str()))?;
        parent.create_dir_all()?;
        if !destination.exists()? {
            source_path.copy_file(&destination)?;
        }
        let destination_path = Path::new(destination.as_str());
        let relative_path = destination_path.strip_prefix(&self.path.as_str())?.to_string_lossy();
        self.index.remember(source_path, &relative_path, id);
        Ok(())
    }

    pub fn newest_item_by_source_path(&self, path: &VfsPath) -> Result<Option<RepositoryItem>> {
        let item = self.index.newest_item_by_source_path(path)?;
        match item {
            None => Ok(None),
            Some(item) => Ok(Some(self.repository_item(&item)?)),
        }
    }

    pub fn item_by_id(&self, id: &ItemId) -> Result<Option<RepositoryItem>> {
        let item = self.index.item_by_id(id)?;
        match item {
            None => Ok(None),
            Some(item) => Ok(Some(self.repository_item(&item)?)),
        }
    }

    pub fn newest_items(&self) -> RepositoryItemIterator {
        RepositoryItemIterator {
            repository: &self,
            iterator: self.index.newest_items(),
        }
    }

    pub fn repository_item(&self, i: &crate::index::item::IndexItem) -> Result<RepositoryItem> {
        let index_item = i.clone();
        let relative_path = index_item.relative_path();
        let repository_path = self.path();
        let original_source_path = index_item.original_source_path();
        let absolute_path = repository_path.join(relative_path)?;
        Ok(RepositoryItem::from(
            &original_source_path,
            &absolute_path,
            relative_path,
            index_item.id(),
            index_item.version(),
        ))
    }

    pub fn data_weight(&self) -> Result<u64> {
        let walkdir = self.data_dir()?.walk_dir()?;
        let total_size = walkdir
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.metadata().ok())
            .filter(|metadata| metadata.file_type == VfsFileType::File)
            .fold(0, |acc, m| acc + m.len);
        Ok(total_size)
    }

    fn data_dir(&self) -> Result<VfsPath> {
        Ok(self.path().join(DATA_DIR_NAME)?)
    }

    fn calculate_id(source_path: &VfsPath) -> Result<ItemId> {
        let source_file = source_path.open_file()?;
        let mut reader = BufReader::new(source_file);
        let mut hasher = Sha512::new();

        io::copy(&mut reader, &mut hasher)?;

        Ok(hasher.finalize()[..].into())
    }
}

#[cfg(test)]
mod must {
    use super::Repository;
    use crate::test::source::TestSource;
    use anyhow::Result;
    use vfs::MemoryFS;

    #[test]
    fn have_size_equal_to_sum_of_sizes_of_backed_up_files() -> Result<()> {
        let file_size1 = 13;
        let file_size2 = 27;
        let source = TestSource::new()?;
        let repository_path = MemoryFS::new().into();
        Repository::init(&repository_path)?;

        let mut backup_repository = Repository::open(&repository_path)?;
        source.write_random_bytes_to_file("file1", file_size1)?;
        backup_repository.store(&source.file_path("file1")?)?;

        source.write_random_bytes_to_file("file2", file_size2)?;

        backup_repository.store(&source.file_path("file2")?)?;

        assert_eq!(file_size1 + file_size2, backup_repository.data_weight()?);
        Ok(())
    }
}

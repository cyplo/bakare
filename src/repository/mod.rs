pub mod item;

use std::{fmt, io};
use std::{
    fmt::{Debug, Formatter},
    path::PathBuf,
};
use std::{fs, path::Path};
use std::{fs::File, io::BufReader};

use crate::index::{Index, IndexItemIterator};
use anyhow::Result;
use anyhow::*;
use item::RepositoryItem;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha512;
use walkdir::WalkDir;

/// represents a place where backup is stored an can be restored from.
/// right now only on-disk directory storage is supported
/// repository always knows the newest version of the index and is responsible for syncing the index to disk
/// and making sure that different threads can access index in parallel
#[derive(Debug)]
pub struct Repository {
    /// path to where the repository is stored on disk
    path: PathBuf,
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
    pub fn init(path: &Path) -> Result<Repository> {
        fs::create_dir_all(path)?;
        let mut index = Index::new()?;
        index.save(path)?;
        let repository = Repository::open(path)?;
        fs::create_dir_all(repository.data_dir()?)?;
        Ok(repository)
    }

    pub fn open(path: &Path) -> Result<Repository> {
        let index = Index::load(path)?;
        let repository = Repository {
            path: path.to_path_buf(),
            index,
        };

        Ok(repository)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save_index(&mut self) -> Result<()> {
        self.index.save(&self.path)
    }

    pub fn store(&mut self, source_path: &Path) -> Result<()> {
        let id = Repository::calculate_id(source_path)?;
        let destination = self.data_dir()?;
        let destination = destination.join(&id.to_string());

        if !source_path.metadata()?.is_file() {
            return Ok(());
        }
        let parent = destination
            .parent()
            .ok_or_else(|| anyhow!("cannot compute parent path for {}", &destination.to_string_lossy()))?;
        fs::create_dir_all(parent)?;
        if !destination.exists() {
            fs::copy(&source_path, &destination)?;
        }
        let relative_path = destination.strip_prefix(&self.path())?;
        self.index.remember(source_path, &relative_path.to_string_lossy(), id);
        Ok(())
    }

    pub fn newest_item_by_source_path(&self, path: &Path) -> Result<Option<RepositoryItem>> {
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
        let absolute_path = repository_path.join(relative_path);
        Ok(RepositoryItem::from(
            &original_source_path,
            &absolute_path,
            relative_path,
            index_item.id(),
            index_item.version(),
        ))
    }

    pub fn data_weight(&self) -> Result<u64> {
        let walker = WalkDir::new(self.data_dir()?);
        let total_size = walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .fold(0, |acc, m| acc + m.len());
        Ok(total_size)
    }

    fn data_dir(&self) -> Result<PathBuf> {
        Ok(self.path().join(DATA_DIR_NAME))
    }

    fn calculate_id(source_path: &Path) -> Result<ItemId> {
        let source_file = File::open(source_path)?;
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
    use tempfile::tempdir;

    #[test]
    fn have_size_equal_to_sum_of_sizes_of_backed_up_files() -> Result<()> {
        let file_size1 = 13;
        let file_size2 = 27;
        let source = TestSource::new()?;
        let repository_path = tempdir()?;
        Repository::init(&repository_path.path())?;

        let mut backup_repository = Repository::open(&repository_path.path())?;
        source.write_random_bytes_to_file("file1", file_size1)?;
        backup_repository.store(&source.file_path("file1")?)?;

        source.write_random_bytes_to_file("file2", file_size2)?;

        backup_repository.store(&source.file_path("file2")?)?;

        assert_eq!(file_size1 + file_size2, backup_repository.data_weight()?);
        Ok(())
    }
}

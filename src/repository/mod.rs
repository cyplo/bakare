pub mod item;

use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

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
pub struct Repository<'a> {
    /// absolute path to where the repository is stored on disk
    path: &'a Path,
    index: Index,
}

const DATA_DIR_NAME: &str = "data";

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize, Hash)]
pub struct ItemId(#[serde(with = "base64")] Vec<u8>);

pub struct RepositoryItemIterator<'a> {
    iterator: IndexItemIterator<'a>,
    index: &'a Index,
}

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
        self.iterator.next().map(|i| self.index.repository_item(&i))
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

impl<'a> Repository<'a> {
    pub fn init(path: &Path) -> Result<()> {
        let mut index = Index::new(path);
        index.save()?;
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Repository> {
        if !path.is_absolute() {
            return Err(anyhow!("path to repository not absolute"));
        }

        let index = Index::load(path)?;
        Ok(Repository { path, index })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save_index(&mut self) -> Result<()> {
        self.index.save()
    }

    pub fn store(&mut self, source_path: &Path) -> Result<()> {
        if !source_path.is_absolute() {
            return Err(anyhow!("path to store not absolute"));
        }
        let id = Repository::calculate_id(source_path)?;
        let destination_path = self.data_dir();
        let destination_path = destination_path.join(id.to_string());
        let destination_path = Path::new(&destination_path);

        if source_path.is_file() {
            let parent = destination_path
                .parent()
                .ok_or_else(|| anyhow!("cannot compute parent path for {}", &destination_path.to_string_lossy()))?;
            fs::create_dir_all(parent)?;
            fs::copy(source_path, destination_path)?;
            let relative_path = destination_path.strip_prefix(self.path)?;
            self.index.remember(source_path, relative_path, id);
        }
        Ok(())
    }

    pub fn newest_item_by_source_path(&self, path: &Path) -> Result<Option<RepositoryItem>> {
        Ok(self
            .index
            .newest_item_by_source_path(path)?
            .map(|i| self.index.repository_item(&i)))
    }

    pub fn item_by_id(&self, id: &ItemId) -> Result<Option<RepositoryItem>> {
        self.index.item_by_id(id).map(|i| i.map(|i| self.index.repository_item(&i)))
    }

    pub fn newest_items(&self) -> RepositoryItemIterator {
        RepositoryItemIterator {
            iterator: self.index.newest_items(),
            index: &self.index,
        }
    }

    pub fn data_weight(&self) -> Result<u64> {
        let total_size = WalkDir::new(self.data_dir())
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.metadata().ok())
            .filter(|metadata| metadata.is_file())
            .fold(0, |acc, m| acc + m.len());
        Ok(total_size)
    }

    fn data_dir(&self) -> PathBuf {
        self.path().join(DATA_DIR_NAME)
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
        let repository_path = tempdir()?.into_path();
        Repository::init(&repository_path)?;

        let mut backup_repository = Repository::open(&repository_path)?;
        source.write_random_bytes_to_file("file1", file_size1)?;
        backup_repository.store(&source.file_path("file1"))?;

        source.write_random_bytes_to_file("file2", file_size2)?;

        backup_repository.store(&source.file_path("file2"))?;

        assert_eq!(file_size1 + file_size2, backup_repository.data_weight()?);
        Ok(())
    }
}

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use walkdir::DirEntry;

use crate::error::BakareError;
use crate::Version;
use crate::Index;

/// represents a place where backup is stored an can be restored from.
/// right now only on-disk directory storage is supported
pub struct Repository<'a> {
    /// absolute path to where the repository is stored on disk
    path: &'a Path,
    index: Index
}

pub struct RepositoryItem {
    version: Version
}

pub struct RepositoryIterator;

impl<'a> Iterator for RepositoryIterator {
    type Item = RepositoryItem;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

impl RepositoryItem {
    pub fn version(&self) -> &Version {
        &self.version
    }
}

impl<'a> Repository<'a> {
    pub fn open(path: &Path) -> Result<Repository, BakareError> {
        // TODO open index from file

        Ok(Repository { path, index: Index {} })
    }

    pub fn iter(&self) -> RepositoryIterator {
        unimplemented!()
    }

    pub fn store(&self, source_path: &Path) -> Result<(), BakareError> {
        // get file id -> contents hash + original path + time of taking notes
        // get storage path for File
        // store file contents
        // remember File

        if source_path.is_dir() {
            fs::create_dir(self.path.join(source_path))?;
        }
        if source_path.is_file() {
            fs::copy(source_path, self.path.join(source_path))?;
        }

        // TODO create new version, remember source_path

        Ok(())
    }

    pub fn item(&self, path: &Path) -> Option<RepositoryItem> {
        unimplemented!()
    }

    pub fn newest_version_for(&self, source_path: &Path) -> Result<Version, BakareError> {
        unimplemented!()
    }

}


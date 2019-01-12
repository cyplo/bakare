use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use walkdir::DirEntry;

use crate::error::BakareError;
use crate::Version;

/// represents a place where backup is stored an can be restored from. E.g. a directory, a cloud service etc
pub struct Repository<'a> {
    path: &'a Path,
}

pub struct StoredItemId;
pub struct RelativePath;
impl<'a> Repository<'a> {
    pub fn new(path: &Path) -> Result<Repository, BakareError> {
        Ok(Repository { path })
    }

    pub fn store_entry(&self, entry: &DirEntry) -> Result<(), BakareError> {
        // get file id -> contents hash + original path + time of taking notes
        // get storage path for File
        // store file contents
        // remember File

        if entry.file_type().is_dir() {
            fs::create_dir(self.path.join(entry.file_name()))?;
        }
        if entry.file_type().is_file() {
            fs::copy(entry.path(), self.path.join(entry.file_name()))?;
        }
        Ok(())
    }

    pub fn newest_version_for(&self, item: &StoredItemId) -> Result<Version, BakareError> {
        unimplemented!()
    }

    pub fn relative_path(&self, path: &str) -> RelativePath {
        unimplemented!()
    }

    pub fn file_id(&self, path: &RelativePath) -> Result<StoredItemId, BakareError> {
        unimplemented!()
    }
}

impl<'a> Iterator for Repository<'a> {
    type Item = StoredItemId;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unimplemented!()
    }
}

impl<'a> Iterator for &Repository<'a> {
    type Item = StoredItemId;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unimplemented!()
    }
}

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::error::BakareError;
use crate::repository::Repository;
use crate::repository::StoredItemId;
use crate::Version;

pub struct Engine<'a> {
    repository: &'a Repository<'a>,
    target_path: &'a Path,
}

impl<'a> Engine<'a> {
    pub fn new(repository: &'a Repository, target_path: &'a Path) -> Self {
        Engine { repository, target_path }
    }

    pub fn restore_all(&self) -> Result<(), BakareError> {
        for item in self.repository {
            self.restore(item)?;
        }
        Ok(())
    }

    fn restore(&self, item: StoredItemId) -> Result<(), BakareError> {
        let version = self.repository.newest_version_for(&item)?;
        self.restore_as_of_version(item, version)
    }

    pub fn restore_as_of_version(&self, what: StoredItemId, version: Version) -> Result<(), BakareError> {
        unimplemented!()
    }

    fn process_entry(&self, entry: &DirEntry) -> Result<(), BakareError> {
        if entry.file_type().is_dir() {
            fs::create_dir(self.target_path.join(entry.file_name()))?;
        }
        if entry.file_type().is_file() {
            fs::copy(entry.path(), self.target_path.join(entry.file_name()))?;
        }
        Ok(())
    }
}

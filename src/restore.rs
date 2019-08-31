use std::fs;
use std::path::Path;

use walkdir::DirEntry;

use crate::error::BakareError;
use crate::repository::Repository;
use crate::repository::RepositoryItem;
use crate::ItemVersion;

pub struct Engine<'a> {
    repository: &'a Repository<'a>,
    target_path: &'a Path,
}

impl<'a> Engine<'a> {
    pub fn new(repository: &'a Repository, target_path: &'a Path) -> Self {
        Engine { repository, target_path }
    }

    pub fn restore_all(&self) -> Result<(), BakareError> {
        for ref item in self.repository.iter() {
            self.restore(item)?;
        }
        Ok(())
    }

    fn restore(&self, item: &RepositoryItem) -> Result<(), BakareError> {
        unimplemented!()
    }

    pub fn restore_as_of_version(&self, item: &RepositoryItem, version: &ItemVersion) -> Result<(), BakareError> {
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

use std::io;
use std::path::Path;
use walkdir::WalkDir;
use walkdir::DirEntry;
use std::fs;

pub struct RestoreEngine<'a> {
    repository_path: &'a Path,
    target_path: &'a Path,
}

pub enum RestoreDescriptor {
    All,
    SpecificPath(String),
}

impl<'a> RestoreEngine<'a> {
    pub fn new(repository_path: &'a Path, target_path: &'a Path) -> Self {
        RestoreEngine {
            repository_path,
            target_path,
        }
    }

    pub fn restore_all(&self) -> Result<(), io::Error> {
        self.restore(RestoreDescriptor::All)
    }

    fn restore(&self, what: RestoreDescriptor) -> Result<(), io::Error> {
        self.restore_as_of_version(what, 0)
    }

    pub fn restore_as_of_version(&self, what: RestoreDescriptor, version: u64) -> Result<(), io::Error> {
        let walker = WalkDir::new(self.repository_path);
        for maybe_entry in walker {
            match maybe_entry {
                Ok(entry) => {
                    if entry.path() != self.repository_path {
                        self.process_entry(&entry)?;
                    }
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    fn process_entry(&self, entry: &DirEntry) -> Result<(), io::Error> {
        if entry.file_type().is_dir() {
            fs::create_dir(self.target_path.join(entry.file_name()))?;
        }
        if entry.file_type().is_file() {
            fs::copy(entry.path(), self.target_path.join(entry.file_name()))?;
        }
        Ok(())
    }
}


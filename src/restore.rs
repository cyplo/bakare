use std::fs;
use std::io;
use std::path::Path;
use storage::Version;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub struct Engine<'a> {
    repository_path: &'a Path,
    target_path: &'a Path,
}

pub enum WhatToRestore {
    All,
    SpecificPath(String),
}

impl<'a> Engine<'a> {
    pub fn new(repository_path: &'a Path, target_path: &'a Path) -> Self {
        Engine {
            repository_path,
            target_path,
        }
    }

    pub fn restore_all(&self) -> Result<(), io::Error> {
        self.restore(WhatToRestore::All)
    }

    fn restore(&self, what: WhatToRestore) -> Result<(), io::Error> {
        self.restore_as_of_version(what, Version(0))
    }

    pub fn restore_as_of_version(&self, what: WhatToRestore, version: Version) -> Result<(), io::Error> {
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

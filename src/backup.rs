use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::error::BakareError;
use crate::repository::Repository;
use crate::RepositoryRelativePath;
use crate::Version;

pub struct Engine<'a> {
    source_path: &'a Path,
    repository: &'a Repository<'a>,
}

impl<'a> Engine<'a> {
    pub fn new(source_path: &'a Path, repository: &'a Repository) -> Self {
        Engine { source_path, repository }
    }

    pub fn backup(&self) -> Result<(), BakareError> {
        let walker = WalkDir::new(self.source_path);
        for maybe_entry in walker {
            let entry = maybe_entry?;
            if entry.path() != self.source_path {
                self.repository.store_entry(&entry)?;
            }
        }
        Ok(())
    }
}

use std::path::Path;

use walkdir::WalkDir;

use crate::error::BakareError;
use crate::repository::Repository;

pub struct Engine<'a> {
    source_path: &'a Path,
    repository: &'a mut Repository<'a>,
}

impl<'a> Engine<'a> {
    pub fn new(source_path: &'a Path, repository: &'a mut Repository<'a>) -> Self {
        Engine { source_path, repository }
    }

    pub fn backup(&mut self) -> Result<(), BakareError> {
        let walker = WalkDir::new(self.source_path);
        for maybe_entry in walker {
            let entry = maybe_entry?;
            if entry.path() != self.source_path {
                self.repository.store(entry.path())?;
            }
        }
        Ok(())
    }
}

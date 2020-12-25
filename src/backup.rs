use anyhow::Result;
use anyhow::*;
use std::path::Path;

use walkdir::WalkDir;

use crate::repository::Repository;

pub struct Engine<'a> {
    source_path: &'a Path,
    repository: &'a mut Repository,
}

impl<'a> Engine<'a> {
    pub fn new(source_path: &'a Path, repository: &'a mut Repository) -> Result<Self> {
        if source_path.ancestors().any(|a| a == repository.path()) {
            return Err(anyhow!("source same as repository"));
        }
        Ok(Engine { source_path, repository })
    }

    pub fn backup(&mut self) -> Result<()> {
        let walker = WalkDir::new(self.source_path);
        let save_every = 16;
        let mut save_counter = 0;
        for maybe_entry in walker {
            let entry = maybe_entry?;
            if entry.path() != self.source_path {
                self.repository.store(entry.path())?;
            }
            save_counter += 1;
            if save_counter == save_every {
                save_counter = 0;
                self.repository.save_index()?;
            }
        }
        self.repository.save_index()?;
        Ok(())
    }
}

use std::path::Path;

use crate::repository::Repository;
use anyhow::Result;
use anyhow::*;
use walkdir::WalkDir;

pub struct Engine<'a> {
    source_path: &'a Path,
    repository: &'a mut Repository,
}

impl<'a> Engine<'a> {
    pub fn new(source_path: &'a Path, repository: &'a mut Repository) -> Result<Self> {
        let mut ancestors = vec![];
        let mut current = Some(source_path.to_path_buf());
        while let Some(path) = current {
            ancestors.push(path.to_path_buf());
            current = path.parent().map(|p| p.to_path_buf());
        }
        if ancestors.into_iter().any(|a| a == repository.path()) {
            return Err(anyhow!("source same as repository"));
        }
        Ok(Engine { source_path, repository })
    }

    pub fn backup(&mut self) -> Result<()> {
        let walker = WalkDir::new(self.source_path);
        for maybe_entry in walker {
            let entry = maybe_entry?;
            if entry.path() != self.source_path {
                self.repository.store(entry.path())?;
            }
        }
        self.repository.save_index()?;
        Ok(())
    }
}

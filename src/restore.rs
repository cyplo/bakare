use std::path::Path;

use crate::repository::{item::RepositoryItem, Repository};
use anyhow::Result;
use anyhow::*;

pub struct Engine<'a> {
    repository: &'a mut Repository,
    target_path: &'a Path,
}

impl<'a> Engine<'a> {
    pub fn new(repository: &'a mut Repository, target_path: &'a Path) -> Result<Self> {
        if !target_path.is_absolute() {
            return Err(anyhow!("path to store not absolute"));
        }
        Ok(Engine { repository, target_path })
    }

    pub fn restore_all(&mut self) -> Result<()> {
        for item in self.repository.newest_items() {
            self.restore(&item)?;
        }
        self.repository.save_index()?;
        Ok(())
    }

    pub fn restore(&self, item: &RepositoryItem) -> Result<()> {
        item.save(self.target_path)?;
        Ok(())
    }
}

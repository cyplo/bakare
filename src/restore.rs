use std::path::Path;

use crate::repository::{item::RepositoryItem, Repository};
use anyhow::Result;

pub struct Engine<'a> {
    repository: &'a mut Repository,
    target_path: &'a Path,
}

impl<'a> Engine<'a> {
    pub fn new(repository: &'a mut Repository, target_path: &'a Path) -> Result<Self> {
        Ok(Engine { repository, target_path })
    }

    pub fn restore_all(&mut self) -> Result<()> {
        let newest_items = self.repository.newest_items();
        for item in newest_items {
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

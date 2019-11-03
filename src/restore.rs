use std::path::Path;

use crate::error::BakareError;
use crate::repository::Repository;
use crate::repository_item::RepositoryItem;

pub struct Engine<'a> {
    repository: &'a mut Repository<'a>,
    target_path: &'a Path,
}

impl<'a> Engine<'a> {
    pub fn new(repository: &'a mut Repository<'a>, target_path: &'a Path) -> Result<Self, BakareError> {
        if !target_path.is_absolute() {
            return Err(BakareError::PathToStoreNotAbsolute);
        }
        Ok(Engine { repository, target_path })
    }

    pub fn restore_all(&mut self) -> Result<(), BakareError> {
        for item in self.repository.newest_items() {
            self.restore(&item)?;
        }
        self.repository.save_index()?;
        Ok(())
    }

    pub fn restore(&self, item: &RepositoryItem) -> Result<(), BakareError> {
        item.save(self.target_path)?;
        Ok(())
    }
}

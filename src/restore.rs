use std::path::Path;

use crate::error::BakareError;
use crate::index::ItemVersion;
use crate::repository::Repository;
use crate::repository_item::RepositoryItem;

pub struct Engine<'a> {
    repository: &'a Repository<'a>,
    target_path: &'a Path,
}

impl<'a> Engine<'a> {
    pub fn new(repository: &'a Repository, target_path: &'a Path) -> Result<Self, BakareError> {
        if !target_path.is_absolute() {
            return Err(BakareError::PathToStoreNotAbsolute);
        }
        Ok(Engine { repository, target_path })
    }

    pub fn restore_all(&self) -> Result<(), BakareError> {
        for item in self.repository.iter() {
            self.restore(&item)?;
        }
        Ok(())
    }

    fn restore(&self, item: &RepositoryItem) -> Result<(), BakareError> {
        println!("restoring {}", item);
        item.save(self.target_path)?;
        Ok(())
    }

    pub fn restore_as_of_version(&self, item: &RepositoryItem, version: &ItemVersion) -> Result<(), BakareError> {
        println!("restoring {}", item);
        Ok(())
    }
}

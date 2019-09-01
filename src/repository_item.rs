use crate::error::BakareError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct RepositoryItem {
    relative_path: Box<Path>,
    absolute_path: Box<Path>,
    original_source_path: Box<Path>,
}

impl RepositoryItem {
    pub fn from(original_source_path: &Path, absolute_path: &Path, relative_path: &Path) -> Self {
        RepositoryItem {
            relative_path: Box::from(relative_path),
            absolute_path: Box::from(absolute_path),
            original_source_path: Box::from(original_source_path),
        }
    }
    pub fn save(&self, save_to: &Path) -> Result<(), BakareError> {
        if !save_to.is_absolute() {
            return Err(BakareError::PathToStoreNotAbsolute);
        }

        let target_path = save_to.join(self.relative_path.clone());
        let parent = target_path.parent().unwrap();
        if !parent.exists() {
            println!("Creating {}", parent.display());
            fs::create_dir_all(parent)?;
        }
        if !self.absolute_path.exists() {
            return Err(BakareError::CorruptedRepoNoFile);
        }
        println!("Saving {} to {}", self.absolute_path.display(), target_path.display());
        fs::copy(self.absolute_path.clone(), target_path.clone())?;

        Ok(())
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub fn original_source_path(&self) -> &Path {
        &self.original_source_path
    }
}

use crate::{repository::ItemId, version::Version};
use anyhow::Result;
use anyhow::*;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::{fmt, fs};

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct RepositoryItem {
    relative_path: Box<Path>,
    absolute_path: Box<Path>,
    original_source_path: Box<Path>,
    id: ItemId,
    version: Version,
}

impl RepositoryItem {
    pub fn from(original_source_path: &Path, absolute_path: &Path, relative_path: &Path, id: ItemId, version: Version) -> Self {
        RepositoryItem {
            relative_path: Box::from(relative_path),
            absolute_path: Box::from(absolute_path),
            original_source_path: Box::from(original_source_path),
            id,
            version,
        }
    }

    pub fn save(&self, save_to: &Path) -> Result<()> {
        if !save_to.is_absolute() {
            return Err(anyhow!("path to store not absolute"));
        }

        let target_path = save_to.join(&self.original_source_path.strip_prefix("/")?);
        if !target_path.is_absolute() {
            return Err(anyhow!("path to store not absolute"));
        }
        let parent = target_path
            .parent()
            .ok_or_else(|| anyhow!("cannot compute parent path for {}", &target_path.to_string_lossy()))?;
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
        if !self.absolute_path.exists() {
            return Err(anyhow!("corrupted repository"));
        }
        fs::copy(&self.absolute_path, &target_path)?;

        Ok(())
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub fn original_source_path(&self) -> &Path {
        &self.original_source_path
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn id(&self) -> &ItemId {
        &self.id
    }
}

impl Display for RepositoryItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "'{}' : {}",
            self.original_source_path().to_string_lossy(),
            hex::encode(self.id())
        )
    }
}
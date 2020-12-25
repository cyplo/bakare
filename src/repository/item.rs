use crate::{repository::ItemId, version::Version};
use anyhow::Result;
use anyhow::*;
use nix::unistd::getpid;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::Path;
use vfs::VfsPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryItem {
    relative_path: String,
    absolute_path: VfsPath,
    original_source_path: String,
    id: ItemId,
    version: Version,
}

impl PartialOrd for RepositoryItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl RepositoryItem {
    pub fn from(
        original_source_path: &str,
        absolute_path: &VfsPath,
        relative_path: &str,
        id: ItemId,
        version: Version,
    ) -> Self {
        RepositoryItem {
            relative_path: relative_path.to_string(),
            absolute_path: absolute_path.clone(),
            original_source_path: original_source_path.to_string(),
            id,
            version,
        }
    }

    pub fn save(&self, save_to: &VfsPath) -> Result<()> {
        let original_source_path = Path::new(self.original_source_path());
        let source_path_relative = original_source_path.strip_prefix("/")?;
        let source_path_relative = source_path_relative.to_string_lossy();
        let target_path = save_to.join(&source_path_relative)?;
        let parent = target_path
            .parent()
            .ok_or_else(|| anyhow!("cannot compute parent path for {}", &target_path.as_str()))?;
        log::debug!("[{}] saving data to {}", getpid(), target_path.as_str());
        parent.create_dir_all()?;
        if !self.absolute_path.exists() {
            return Err(anyhow!("corrupted repository"));
        }
        self.absolute_path.copy_file(&target_path)?;

        Ok(())
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    pub fn original_source_path(&self) -> &str {
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
        write!(f, "'{}' : {}", self.original_source_path(), hex::encode(self.id()))
    }
}

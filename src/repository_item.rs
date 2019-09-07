use crate::error::BakareError;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::{fmt, fs};

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct RepositoryItem {
    relative_path: Box<Path>,
    absolute_path: Box<Path>,
    original_source_path: Box<Path>,
    version: Box<[u8]>,
}

impl RepositoryItem {
    pub fn from(original_source_path: &Path, absolute_path: &Path, relative_path: &Path, version: Box<[u8]>) -> Self {
        RepositoryItem {
            relative_path: Box::from(relative_path),
            absolute_path: Box::from(absolute_path),
            original_source_path: Box::from(original_source_path),
            version,
        }
    }

    pub fn save(&self, save_to: &Path) -> Result<(), BakareError> {
        if !save_to.is_absolute() {
            return Err(BakareError::PathToStoreNotAbsolute);
        }

        let target_path = save_to.join(&self.relative_path);
        if !target_path.is_absolute() {
            return Err(BakareError::PathToStoreNotAbsolute);
        }
        let parent = target_path.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
        if !self.absolute_path.exists() {
            return Err(BakareError::CorruptedRepoNoFile);
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

    pub fn version(&self) -> Box<[u8]> {
        self.version.clone()
    }
}

impl Display for RepositoryItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "'{}' : {}",
            self.original_source_path().to_string_lossy(),
            hex::encode(self.version())
        )
    }
}

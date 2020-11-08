use serde::{Deserialize, Serialize};

use crate::repository::ItemId;
use crate::{repository_item::RepositoryItem, version::Version};

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize)]
pub struct IndexItem {
    relative_path: String,
    original_source_path: String,
    id: ItemId,
    version: Version,
}

impl IndexItem {
    pub fn from(original_source_path: String, relative_path: String, id: ItemId, version: Version) -> IndexItem {
        IndexItem {
            relative_path,
            original_source_path,
            id,
            version,
        }
    }

    pub fn next_version(&self, id: ItemId, relative_path: String) -> IndexItem {
        IndexItem {
            original_source_path: self.original_source_path.clone(),
            version: self.version.next(),
            relative_path,
            id,
        }
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn id(&self) -> ItemId {
        self.id.clone()
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    pub fn original_source_path(&self) -> &str {
        &self.original_source_path
    }
}

impl From<RepositoryItem> for IndexItem {
    fn from(i: RepositoryItem) -> Self {
        IndexItem {
            relative_path: i.relative_path().to_string_lossy().to_string(),
            original_source_path: i.original_source_path().to_string_lossy().to_string(),
            id: i.id().clone(),
            version: *i.version(),
        }
    }
}

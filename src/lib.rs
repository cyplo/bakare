use crate::repository::RepositoryItem;

pub mod backup;
pub mod error;
pub mod restore;
pub mod source;

pub mod repository;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct ItemVersion<'a>(&'a str);

#[derive(Copy, Clone)]
pub struct IndexVersion;

struct IndexViewReadonly<'a> {
    index_version: IndexVersion,
    items: Vec<RepositoryItem<'a>>,
}

use std::cmp::Ordering;
use std::path::Path;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Version(pub u64);

struct Index;

impl Index {
    fn new() -> Self {
        Self {}
    }

    fn store(&mut self, path: &str, hash: &[u8]) -> (Version, String) {
        (Version(0), "".to_string())
    }

    fn version(&self, hash: &[u8]) -> Version {
        Version(0)
    }
}

#[cfg(test)]
mod should {

    use super::*;

    #[test]
    fn support_file_versions() {
        let mut index = Index::new();
        let (v1, _) = index.store("/some/path", "some hash".as_bytes());
        let (v2, _) = index.store("/some/path", "some other hash".as_bytes());

        assert_eq!(v1, index.version("some hash".as_bytes()));
        assert_eq!(v2, index.version("some other hash".as_bytes()));

        assert!(v2 > v1);
    }

    #[test]
    fn support_deduplication() {
        let mut index = Index::new();
        let (_, storage_path1) = index.store("/some/path", "same hash".as_bytes());
        let (_, storage_path2) = index.store("/some/path", "same hash".as_bytes());
        let (_, storage_path3) = index.store("/some/other/path", "same hash".as_bytes());

        assert_eq!(storage_path1, storage_path2);
        assert_ne!(storage_path1, storage_path3);
    }
}

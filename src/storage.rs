use std::cmp::Ordering;
use std::path::Path;

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct Version(pub u64);

struct Index;

impl Index {
    fn new() -> Self {
        Self {}
    }

    fn store(&mut self, path: &Path, hash: &[u8]) -> Version {
        Version(0)
    }
}

#[cfg(test)]
mod should {

    use super::*;

    #[test]
    fn support_file_versions() {
        // put path and hash into index -> v0
        // put same path different hash -> v1
        // query for v0, v1
        let mut index = Index::new();
        let v1 = index.store(Path::new("/some/path"), "some hash".as_bytes());
        let v2 = index.store(Path::new("/some/path"), "some other hash".as_bytes());

        assert!(v2 > v1);
    }

    #[test]
    fn support_deduplication() {
        // put path and hash into index
        // put same hash, different path
        // should get same storage paths
    }
}

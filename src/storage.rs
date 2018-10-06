use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Version(pub u64);

impl Version {
    fn next(self) -> Self {
        Version(self.0 + 1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Hash([u8; 32]);

#[derive(Clone)]
struct IndexHashEntry {
    source_paths: HashSet<String>,
    storage_path: String,
}

#[derive(Clone)]
struct IndexPathEntry {
    hash: Hash,
    version: Version,
    storage_path: String,
}

struct Index<'a> {
    file_hashes: HashMap<Hash, IndexHashEntry>,
    file_paths: HashMap<&'a str, IndexPathEntry>,
}

impl<'a> Index<'a> {
    fn new() -> Self {
        Self {
            file_hashes: HashMap::new(),
            file_paths: HashMap::new()
        }
    }

    fn store(&mut self, source_path: &'a str, hash: Hash) -> (Version, String) {
        let path_entry = {
            self.file_paths.get(source_path).map_or_else(
                || IndexPathEntry {
                    hash,
                    version: Version(0),
                    storage_path: format!("{:X?}", hash.0),
                },
                |old_entry| {
                    if old_entry.hash == hash {
                        old_entry.clone() // FIXME optimise
                    } else {
                        IndexPathEntry {
                            hash,
                            version: old_entry.version.next(),
                            storage_path: old_entry.storage_path.clone(),
                        }
                    }
                },
            )
        };

        self.file_paths.insert(source_path, path_entry.clone());

        let hash_entry = {
            self.file_hashes.get(&hash).map_or_else(
                || {
                    let mut source_paths = HashSet::new();
                    source_paths.insert(source_path.to_string());
                    IndexHashEntry {
                        source_paths,
                        storage_path: format!("{:X?}", hash.0),
                    }
                },
                |old_entry| {
                    let mut source_paths = old_entry.source_paths.clone();
                    source_paths.insert(source_path.to_string());
                    IndexHashEntry {
                        source_paths,
                        storage_path: old_entry.storage_path.clone()
                    }
                },
            )
        };

        self.file_hashes.insert(hash, hash_entry);
        (path_entry.version, path_entry.storage_path.to_string())
    }

    fn latest_version_for_path(&self, path: &str) -> Option<Version> {
        self.file_paths.get(path).map_or(None, |entry| Some(entry.version))
    }
}

#[cfg(test)]
mod should {

    use super::*;

    const SOME_HASH: Hash = Hash([1; 32]);
    const SOME_OTHER_HASH: Hash = Hash([2; 32]);

    #[test]
    fn support_file_versions() {
        let mut index = Index::new();
        let (v1, _) = index.store("/some/path", SOME_HASH);
        let (v2, _) = index.store("/some/path", SOME_OTHER_HASH);

        assert_eq!(v2, index.latest_version_for_path("/some/path").unwrap());

        assert!(v2 > v1);
    }

    #[test]
    fn support_deduplication() {
        let mut index = Index::new();
        let (_, storage_path1) = index.store("/some/path", SOME_HASH);
        let (_, storage_path2) = index.store("/some/path", SOME_HASH);
        let (_, storage_path3) = index.store("/some/other/path", SOME_HASH);

        assert_eq!(storage_path1, storage_path2);
        assert_eq!(storage_path1, storage_path3);
    }
}

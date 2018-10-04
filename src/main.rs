extern crate core;
#[cfg(test)]
extern crate dir_diff;
#[cfg(test)]
extern crate tempfile;
extern crate walkdir;

use std::fs;
use std::io;
use std::path::Path;
use walkdir::DirEntry;
use walkdir::WalkDir;

struct BackupEngine<'a> {
    source_path: &'a Path,
    repository_path: &'a Path,
}

impl<'a> BackupEngine<'a> {
    fn new(source_path: &'a Path, repository_path: &'a Path) -> Self {
        BackupEngine {
            source_path,
            repository_path,
        }
    }

    fn backup(&self) -> Result<(), io::Error> {
        let walker = WalkDir::new(self.source_path);
        for maybe_entry in walker {
            match maybe_entry {
                Ok(entry) => {
                    if entry.path() != self.source_path {
                        self.process_entry(&entry)?;
                    }
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    fn file_version(&self, path: &Path) -> u64 {
        0
    }

    fn process_entry(&self, entry: &DirEntry) -> Result<(), io::Error> {
        if entry.file_type().is_dir() {
            fs::create_dir(self.repository_path.join(entry.file_name()))?;
        }
        if entry.file_type().is_file() {
            fs::copy(entry.path(), self.repository_path.join(entry.file_name()))?;
        }
        Ok(())
    }
}

struct RestoreEngine<'a> {
    repository_path: &'a Path,
    target_path: &'a Path,
}

enum RestoreDescriptor {
    All,
    SpecificPath(String)
}

impl<'a> RestoreEngine<'a> {
    fn new(repository_path: &'a Path, target_path: &'a Path) -> Self {
        RestoreEngine {
            repository_path,
            target_path,
        }
    }

    fn restore_all(&self) -> Result<(), io::Error> {
        self.restore(RestoreDescriptor::All)
    }

    fn restore(&self, what: RestoreDescriptor) -> Result<(), io::Error> {
        self.restore_as_of_version(what, 0)
    }

    fn restore_as_of_version(&self, what: RestoreDescriptor, version: u64) -> Result<(), io::Error> {
        let walker = WalkDir::new(self.repository_path);
        for maybe_entry in walker {
            match maybe_entry {
                Ok(entry) => {
                    if entry.path() != self.repository_path {
                        self.process_entry(&entry)?;
                    }
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    fn process_entry(&self, entry: &DirEntry) -> Result<(), io::Error> {
        if entry.file_type().is_dir() {
            fs::create_dir(self.target_path.join(entry.file_name()))?;
        }
        if entry.file_type().is_file() {
            fs::copy(entry.path(), self.target_path.join(entry.file_name()))?;
        }
        Ok(())
    }
}

fn main() {}

mod rustback {

    #[cfg(test)]
    mod should {

        use dir_diff::is_different;
        use std::fs::File;
        use std::io::Error;
        use std::io::Write;
        use tempfile::tempdir;
        use BackupEngine;
        use RestoreEngine;
        use std::path::Path;
        use RestoreDescriptor;
        use std::io::Read;

        #[test]
        fn restore_backed_up_files() -> Result<(), Error> {
            let source = tempdir()?;
            let repository = tempdir()?;

            File::create(source.path().join("first"))?.write_all("some contents".as_bytes())?;
            File::create(source.path().join("second"))?.write_all("some contents".as_bytes())?;
            File::create(source.path().join("third"))?.write_all("some other contents".as_bytes())?;

            is_same_after_restore(source.path(), repository.path())
        }

        #[test]
        fn restore_older_version_of_file() -> Result<(), Error> {
            let source = tempdir()?;
            let repository = tempdir()?;
            let backup_engine = BackupEngine::new(source.path(), repository.path());
            let path = "first";
            let new_file_contents = "totally new contents";
            let restore_target = tempdir()?;
            let restore_engine = RestoreEngine::new(repository.path(), &restore_target.path());
            let old_contents = "some contents";

            File::create(source.path().join(path))?.write_all(old_contents.as_bytes())?;
            backup_engine.backup()?;
            let old_version = backup_engine.file_version(path.as_ref());
            File::create(source.path().join(path))?.write_all(new_file_contents.as_bytes())?;
            backup_engine.backup()?;

            restore_engine.restore_as_of_version(RestoreDescriptor::SpecificPath(path.into()), old_version)?;

            let restored_path = restore_target.path().join(path);
            let mut actual_contents = String::new();
            File::open(restored_path)?.read_to_string(&mut actual_contents)?;

            assert_eq!(old_contents, actual_contents);

            Ok(())
        }

        fn is_same_after_restore(source_path: &Path, repository_path: &Path) -> Result<(), Error> {
            let backup_engine = BackupEngine::new(source_path, repository_path);
            backup_engine.backup()?;

            let restore_target = tempdir()?;
            let restore_engine = RestoreEngine::new(repository_path, &restore_target.path());
            restore_engine.restore_all()?;

            let are_source_and_target_different = is_different(source_path, &restore_target.path()).unwrap();
            assert!(!are_source_and_target_different);
            Ok(())
        }
    }

}

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

impl<'a> RestoreEngine<'a> {
    fn new(repository_path: &'a Path, target_path: &'a Path) -> Self {
        RestoreEngine {
            repository_path,
            target_path,
        }
    }

    fn restore(&self) -> Result<(), io::Error> {
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

        #[test]
        fn restore_backed_up_files() -> Result<(), Error> {
            let source = tempdir()?;

            File::create(source.path().join("first"))?.write_all("some contents".as_bytes())?;
            File::create(source.path().join("second"))?.write_all("some contents".as_bytes())?;
            File::create(source.path().join("third"))?.write_all("some other contents".as_bytes())?;

            let repository = tempdir()?;

            is_same_after_restore(source.path(), repository.path())
        }


        fn is_same_after_restore(source_path: &Path, repository_path: &Path) -> Result<(), Error> {
            let backup_engine = BackupEngine::new(source_path, repository_path);
            backup_engine.backup()?;

            let restore_target = tempdir()?;
            let restore_engine = RestoreEngine::new(repository_path, &restore_target.path());
            restore_engine.restore()?;

            let are_source_and_target_different = is_different(source_path, &restore_target.path()).unwrap();
            assert!(!are_source_and_target_different);
            Ok(())
        }
    }

}

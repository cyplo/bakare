extern crate crypto;
extern crate dir_diff;
extern crate tempfile;
extern crate walkdir;

use std::path::Path;
use walkdir::DirEntry;
use walkdir::Error;
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

    fn backup(&self) -> Result<(), Error> {
        let walker = WalkDir::new(self.source_path);
        for maybe_entry in walker {
            match maybe_entry {
                Ok(entry) => self.process_entry(entry),
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn process_entry(&self, entry: DirEntry) {
        println!("{:?}", entry.path());
    }
}

struct RestoreEngine;
impl RestoreEngine {
    fn new(repository_path: &Path, target_path: &Path) -> Self {
        RestoreEngine {}
    }

    fn restore(&self) {}
}

mod rustback {

    use super::*;

    #[cfg(test)]
    mod should {

        use super::*;
        use dir_diff::is_different;
        use std::fs::write;
        use std::fs::File;
        use std::io::Error;
        use std::io::{self, Write};
        use tempfile::tempdir;
        use tempfile::tempfile_in;
        use tempfile::TempDir;

        #[test]
        fn be_able_to_restore_backed_up_files() -> Result<(), Error> {
            let source = tempdir()?;

            File::create(source.path().join("first"))?.write_all("some contents".as_bytes())?;
            File::create(source.path().join("second"))?.write_all("some contents".as_bytes())?;
            File::create(source.path().join("third"))?.write_all("some other contents".as_bytes())?;

            let repository = tempdir()?;
            let backup_engine = BackupEngine::new(&source.path(), &repository.path());
            backup_engine.backup();

            let restore_target = tempdir()?;
            let restore_engine = RestoreEngine::new(&repository.path(), &restore_target.path());
            restore_engine.restore();

            let are_source_and_target_different =
                is_different(&source.path(), &restore_target.path()).unwrap();
            assert!(!are_source_and_target_different);
            Ok(())
        }
    }

}

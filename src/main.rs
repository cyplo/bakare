extern crate crypto;
extern crate tempfile;
extern crate dir_diff;

use std::path::Path;

struct BackupEngine;
impl BackupEngine {
    fn new(source_path: &Path, repository_path: &Path) -> Self {
        BackupEngine {}
    }

    fn backup(&self) {}
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
        use std::fs::File;
        use std::io::Error;
        use std::io::{self, Write};
        use tempfile::tempdir;
        use tempfile::tempfile_in;
        use tempfile::TempDir;
        use dir_diff::is_different;
        use std::fs::write;

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

            let is_source_and_destination_different = is_different(&source.path(), &restore_target.path()).unwrap();
            assert!(!is_source_and_destination_different);
            Ok(())
        }
    }

}


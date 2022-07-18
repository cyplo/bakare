use anyhow::Result;
use anyhow::*;
use fail::fail_point;
use std::{
    fs::{remove_file, File},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};
use uuid::Uuid;
use walkdir::WalkDir;

use rand::{rngs::OsRng, RngCore};
use std::{thread, time};

pub struct Lock {
    path: PathBuf,
}

const MAX_TIMEOUT_MILLIS: u16 = 8192;
const FILE_EXTENSION: &str = ".lock";

impl Lock {
    #[allow(clippy::self_named_constructors)]
    pub fn lock(index_directory: &Path) -> Result<Self> {
        Lock::lock_with_timeout(index_directory, MAX_TIMEOUT_MILLIS)
    }

    pub fn lock_with_timeout(index_directory: &Path, max_timeout_millis: u16) -> Result<Self> {
        let mut buffer = [0u8; 16];
        OsRng.fill_bytes(&mut buffer);
        let id = Uuid::from_bytes(buffer);
        Lock::wait_to_have_sole_lock(id, index_directory, max_timeout_millis)?;
        let path = Lock::lock_file_path(index_directory, id)?;
        Ok(Lock { path })
    }

    pub fn release(self) -> Result<()> {
        self.delete_lock_file()?;
        Ok(())
    }

    fn delete_lock_file(&self) -> Result<()> {
        if self.path.exists() {
            remove_file(&self.path)?;
        }
        Ok(())
    }

    fn wait_to_have_sole_lock(lock_id: Uuid, index_directory: &Path, max_timeout_millis: u16) -> Result<()> {
        let start_time = Instant::now();
        let _ = Lock::create_lock_file(lock_id, index_directory);
        while !Lock::sole_lock(lock_id, index_directory)? {
            let path = Lock::lock_file_path(index_directory, lock_id)?;
            if path.exists() {
                remove_file(path)?;
            }
            let sleep_duration = time::Duration::from_millis((OsRng.next_u32() % 64).into());
            thread::sleep(sleep_duration);

            // timeout will take care of permanent errors
            let _ = Lock::create_lock_file(lock_id, index_directory);

            if start_time.elapsed().as_millis() > max_timeout_millis.into() {
                return Err(anyhow!("timed out waiting on lock"));
            }
        }
        Ok(())
    }

    fn sole_lock(lock_id: Uuid, index_directory: &Path) -> Result<bool> {
        let my_lock_file_path = Lock::lock_file_path(index_directory, lock_id)?;

        let all_locks_count =
            Lock::count_files(|e| e.file_name().to_string_lossy().ends_with(FILE_EXTENSION), index_directory)?;
        if all_locks_count != 1 {
            return Ok(false);
        }
        let my_locks_count = Lock::count_files(|e| e.path() == my_lock_file_path, index_directory)?;
        if my_locks_count != 1 {
            return Ok(false);
        }
        Ok(true)
    }

    fn count_files<P>(predicate: P, directory: &Path) -> Result<usize>
    where
        P: Fn(&walkdir::DirEntry) -> bool,
    {
        let walker = WalkDir::new(directory);
        let matching = walker.into_iter().filter_map(|e| e.ok()).filter(predicate);
        Ok(matching.count())
    }

    fn create_lock_file(lock_id: Uuid, index_directory: &Path) -> Result<()> {
        let lock_file_path = Lock::lock_file_path(index_directory, lock_id)?;
        fail_point!("create-lock-file", |e: Option<String>| Err(anyhow!(e.unwrap())));
        let mut file = File::create(lock_file_path)?;
        let lock_id_text = lock_id.as_hyphenated().to_string();
        let lock_id_bytes = lock_id_text.as_bytes();
        Ok(file.write_all(lock_id_bytes)?)
    }

    fn lock_file_path(path: &Path, lock_id: Uuid) -> Result<PathBuf> {
        let file_name = format!("{}{}", lock_id, FILE_EXTENSION);
        Ok(path.join(&file_name))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.delete_lock_file();
    }
}

#[cfg(test)]
mod must {
    use super::Lock;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use tempfile::tempdir;
    #[cfg(feature = "failpoints")]
    use two_rusty_forks::rusty_fork_test;

    #[test]
    fn be_released_when_dropped() -> Result<()> {
        let temp_dir = tempdir()?;
        let initial_number_of_entries = temp_dir.path().read_dir()?.count();
        {
            let _lock = Lock::lock(temp_dir.path())?;
        }
        let entries = temp_dir.path().read_dir()?.count();

        assert_eq!(entries, initial_number_of_entries);
        Ok(())
    }

    #[cfg(feature = "failpoints")]
    rusty_fork_test! {
        #[test]
        fn be_able_to_lock_when_creating_lock_file_fails_sometimes() {
            fail::cfg("create-lock-file", "90%10*return(some lock file creation error)->off").unwrap();
            let temp_dir = tempdir().unwrap();

            let lock = Lock::lock(temp_dir.path()).unwrap();
            lock.release().unwrap();
        }
    }

    #[cfg(feature = "failpoints")]
    rusty_fork_test! {
        #[test]
        fn know_to_give_up_when_creating_lock_file_always_fails()  {
            fail::cfg("create-lock-file", "return(persistent lock file creation error)").unwrap();
            let temp_dir = tempdir().unwrap();

            assert!(Lock::lock_with_timeout(temp_dir.path(), 1).is_err());
        }
    }
}

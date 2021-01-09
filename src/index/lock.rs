use anyhow::Result;
use anyhow::*;
use fail::fail_point;
use std::{io::Write, time::Instant};
use uuid::Uuid;
use vfs::VfsPath;

use rand::{rngs::OsRng, RngCore};
use std::{thread, time};

pub struct Lock {
    path: VfsPath,
}

const MAX_TIMEOUT_MILLIS: u16 = 8192;
const FILE_EXTENSION: &str = ".lock";

impl Lock {
    pub fn lock(index_directory: &VfsPath) -> Result<Self> {
        Lock::lock_with_timeout(index_directory, MAX_TIMEOUT_MILLIS)
    }

    pub fn lock_with_timeout(index_directory: &VfsPath, max_timeout_millis: u16) -> Result<Self> {
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
            self.path.remove_file()?;
        }
        Ok(())
    }

    fn wait_to_have_sole_lock(lock_id: Uuid, index_directory: &VfsPath, max_timeout_millis: u16) -> Result<()> {
        let start_time = Instant::now();
        let _ = Lock::create_lock_file(lock_id, index_directory);
        while !Lock::sole_lock(lock_id, index_directory)? {
            let path = Lock::lock_file_path(index_directory, lock_id)?;
            if path.exists() {
                path.remove_file()?;
            }
            let sleep_duration = time::Duration::from_millis((OsRng.next_u32() % 64).into());
            thread::sleep(sleep_duration);
            let _ = Lock::create_lock_file(lock_id, index_directory);
            if start_time.elapsed().as_millis() > max_timeout_millis.into() {
                return Err(anyhow!("timed out waiting on lock"));
            }
        }
        Ok(())
    }

    fn sole_lock(lock_id: Uuid, index_directory: &VfsPath) -> Result<bool> {
        let my_lock_file_path = Lock::lock_file_path(index_directory, lock_id)?;
        let locks = Lock::all_locks(index_directory)?;
        let mut only_mine = true;
        for path in &locks {
            if path != &my_lock_file_path {
                only_mine = false;
                break;
            }
        }
        if locks.iter().count() == 0 {
            return Ok(false);
        }
        Ok(only_mine)
    }

    fn all_locks(index_directory: &VfsPath) -> Result<Vec<VfsPath>> {
        Ok(index_directory
            .read_dir()?
            .into_iter()
            .filter(|f| f.filename().ends_with(FILE_EXTENSION))
            .collect())
    }

    fn create_lock_file(lock_id: Uuid, index_directory: &VfsPath) -> Result<()> {
        let lock_file_path = Lock::lock_file_path(index_directory, lock_id)?;
        fail_point!("create-lock-file", |e: Option<String>| Err(anyhow!(e.unwrap())));
        let mut file = lock_file_path.create_file()?;
        let lock_id_text = lock_id.to_hyphenated().to_string();
        let lock_id_bytes = lock_id_text.as_bytes();
        Ok(file.write_all(lock_id_bytes)?)
    }

    fn lock_file_path(path: &VfsPath, lock_id: Uuid) -> Result<VfsPath> {
        let file_name = format!("{}.{}", lock_id, FILE_EXTENSION);
        Ok(path.join(&file_name)?)
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

    #[cfg(feature = "failpoints")]
    use two_rusty_forks::rusty_fork_test;
    use vfs::{MemoryFS, VfsPath};

    #[test]
    fn be_released_when_dropped() -> Result<()> {
        let temp_dir: VfsPath = MemoryFS::new().into();
        {
            let _lock = Lock::lock(&temp_dir);
        }
        let entries = temp_dir.read_dir()?.count();

        assert_eq!(entries, 0);
        Ok(())
    }

    #[cfg(feature = "failpoints")]
    rusty_fork_test! {
        #[test]
        fn be_able_to_lock_when_creating_lock_file_fails_sometimes() {
            fail::cfg("create-lock-file", "90%10*return(some lock file creation error)->off").unwrap();
            let path = MemoryFS::new().into();

            let lock = Lock::lock(&path).unwrap();
            lock.release().unwrap();
        }
    }

    #[cfg(feature = "failpoints")]
    rusty_fork_test! {
        #[test]
        fn know_to_give_up_when_creating_lock_file_always_fails()  {
            fail::cfg("create-lock-file", "return(persistent lock file creation error)").unwrap();
            let path = MemoryFS::new().into();

            assert!(Lock::lock_with_timeout(&path, 1).is_err());
        }
    }
}

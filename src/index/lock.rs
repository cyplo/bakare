use anyhow::Result;
use std::io::Write;
use uuid::Uuid;
use vfs::VfsPath;

use rand::{rngs::OsRng, RngCore};
use std::{thread, time};

pub struct Lock {
    path: VfsPath,
}

impl Lock {
    pub fn lock(index_directory: &VfsPath) -> Result<Self> {
        let mut buffer = [0u8; 16];
        OsRng.fill_bytes(&mut buffer);
        let id = Uuid::from_bytes(buffer);
        Lock::wait_to_have_sole_lock(id, index_directory)?;
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

    fn wait_to_have_sole_lock(lock_id: Uuid, index_directory: &VfsPath) -> Result<()> {
        Lock::create_lock_file(lock_id, index_directory)?;
        while !Lock::sole_lock(lock_id, index_directory)? {
            let path = Lock::lock_file_path(index_directory, lock_id)?;
            path.remove_file()?;
            let sleep_duration = time::Duration::from_millis((OsRng.next_u32() % 256).into());
            thread::sleep(sleep_duration);
            Lock::create_lock_file(lock_id, index_directory)?;
        }
        Ok(())
    }

    fn sole_lock(lock_id: Uuid, index_directory: &VfsPath) -> Result<bool> {
        let my_lock_file_path = Lock::lock_file_path(index_directory, lock_id)?;
        let locks = Lock::all_locks(index_directory)?;
        let mut only_mine = true;
        for path in locks {
            if path != my_lock_file_path {
                only_mine = false;
                break;
            }
        }
        Ok(only_mine)
    }

    fn all_locks(index_directory: &VfsPath) -> Result<Vec<VfsPath>> {
        Ok(index_directory
            .read_dir()?
            .into_iter()
            .filter(|f| f.filename().ends_with(".lock"))
            .collect())
    }

    fn create_lock_file(lock_id: Uuid, index_directory: &VfsPath) -> Result<()> {
        let lock_file_path = Lock::lock_file_path(index_directory, lock_id)?;
        let mut file = lock_file_path.create_file()?;
        let lock_id_text = lock_id.to_hyphenated().to_string();
        let lock_id_bytes = lock_id_text.as_bytes();
        Ok(file.write_all(lock_id_bytes)?)
    }

    fn lock_file_path(path: &VfsPath, lock_id: Uuid) -> Result<VfsPath> {
        let file_name = format!("{}.lock", lock_id);
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
}

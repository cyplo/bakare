use anyhow::Result;
use anyhow::*;
use atomicwrites::{AtomicFile, DisallowOverwrite};
use glob::{glob, Paths};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use rand::{rngs::OsRng, RngCore};
use std::{thread, time};

pub struct Lock {
    path: PathBuf,
}

impl Lock {
    pub fn new(index_directory: &Path) -> Result<Self> {
        let mut buffer = [0u8; 16];
        OsRng.fill_bytes(&mut buffer);
        let id = Uuid::from_bytes(buffer);
        Lock::wait_to_have_sole_lock(id, index_directory)?;
        let path = Lock::lock_file_path(index_directory, id);
        Ok(Lock { path })
    }

    pub fn release(self) -> Result<()> {
        self.delete_lock_file()?;
        Ok(())
    }

    fn delete_lock_file(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    fn wait_to_have_sole_lock(lock_id: Uuid, index_directory: &Path) -> Result<()> {
        Lock::create_lock_file(lock_id, index_directory)?;
        while !Lock::sole_lock(lock_id, index_directory)? {
            let path = Lock::lock_file_path(index_directory, lock_id);
            std::fs::remove_file(path)?;
            let sleep_duration = time::Duration::from_millis((OsRng.next_u32() % 256).into());
            thread::sleep(sleep_duration);
            Lock::create_lock_file(lock_id, index_directory)?;
        }
        Ok(())
    }

    fn sole_lock(lock_id: Uuid, index_directory: &Path) -> Result<bool> {
        let my_lock_file_path = Lock::lock_file_path(index_directory, lock_id);
        let locks = Lock::all_locks(index_directory)?;
        let mut only_mine = true;
        for path in locks {
            let path = path?;
            if path.to_string_lossy() != my_lock_file_path.to_string_lossy() {
                only_mine = false;
                break;
            }
        }
        Ok(only_mine)
    }

    fn all_locks(index_directory: &Path) -> Result<Paths> {
        let locks_glob = Lock::locks_glob(index_directory);
        Ok(glob(&locks_glob)?)
    }

    fn create_lock_file(lock_id: Uuid, index_directory: &Path) -> Result<()> {
        let lock_file_path = Lock::lock_file_path(index_directory, lock_id);
        let file = AtomicFile::new(lock_file_path, DisallowOverwrite);
        match file.write(|f| f.write_all(lock_id.to_hyphenated().to_string().as_bytes())) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("error acquiring lock: {}", e)),
        }
    }

    fn lock_file_path(path: &Path, lock_id: Uuid) -> PathBuf {
        let path_text = &format!("{}/{}.lock", path.to_string_lossy(), lock_id);
        Path::new(path_text).to_path_buf()
    }

    fn locks_glob(path: &Path) -> String {
        format!("{}/*.lock", path.to_string_lossy())
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        match self.delete_lock_file() {
            _ => (),
        }
    }
}

#[cfg(test)]
mod must {
    use super::Lock;
    use anyhow::Result;
    use std::{fs, io};

    #[test]
    fn be_released_when_dropped() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        {
            let _lock = Lock::new(&temp_dir.path());
        }
        let entries = fs::read_dir(temp_dir.into_path())?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        assert_eq!(entries.len(), 0);
        Ok(())
    }
}

use std::fs::File;
use std::path::Path;
use std::time::Duration;
use std::{fs, thread};

use uuid::Uuid;

use glob::glob;

use crate::error::BakareError;

pub fn release_lock(path: &Path, lock_id: Uuid) -> Result<(), BakareError> {
    let lock_file_path = lock_file_path(path, lock_id);
    fs::remove_file(lock_file_path)?;
    Ok(())
}

pub fn acquire_lock(lock_id: Uuid, index_directory: &Path) -> Result<(), BakareError> {
    let lock_file_path = lock_file_path(index_directory, lock_id);
    wait_for_only_my_locks_left(lock_id, index_directory)?;
    File::create(lock_file_path)?;
    Ok(())
}

fn lock_file_path(path: &Path, lock_id: Uuid) -> String {
    format!("{}/{}.lock", path.to_string_lossy(), lock_id)
}

fn wait_for_only_my_locks_left(lock_id: Uuid, index_directory: &Path) -> Result<(), BakareError> {
    let parent_directory_path = index_directory.to_string_lossy();
    let lock_file_extension = "lock";
    let my_lock_file_path = format!("{}/{}.{}", parent_directory_path, lock_id, lock_file_extension);

    loop {
        let mut locks = glob(&format!("{}/*.{}", parent_directory_path, lock_file_extension))?;
        {
            let only_my_locks = locks.all(|path| match path {
                Ok(path) => path.to_string_lossy() == my_lock_file_path,
                Err(_) => false,
            });
            if only_my_locks {
                break;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

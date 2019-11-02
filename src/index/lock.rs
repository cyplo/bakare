use std::fs::File;
use std::path::Path;
use std::time::Duration;
use std::{fs, thread};

use uuid::Uuid;

use glob::glob;

use crate::error::BakareError;

pub fn release_lock(path: &Path, lock_id: Uuid) -> Result<(), BakareError> {
    let lock_file_path = lock_file_path(path, lock_id);
    delete_lock_file(lock_file_path)?;
    Ok(())
}

fn delete_lock_file(lock_file_path: String) -> Result<(), BakareError> {
    fs::remove_file(lock_file_path.clone()).map_err(|e| (e, lock_file_path))?;
    Ok(())
}

pub fn acquire_lock(lock_id: Uuid, index_directory: &Path) -> Result<(), BakareError> {
    let lock_file_path = lock_file_path(index_directory, lock_id);
    let mut just_me = false;
    let mut free = false;
    while !(free && just_me) {
        free = wait_for_only_my_locks_left(lock_id, index_directory)?;
        if free {
            create_lock_file(lock_file_path.clone())?;
        }
        just_me = wait_for_only_my_locks_left(lock_id, index_directory)?;
        if !just_me {
            delete_lock_file(lock_file_path.clone())?;
        }
    }
    Ok(())
}

fn create_lock_file(lock_file_path: String) -> Result<(), BakareError> {
    File::create(lock_file_path.clone()).map_err(|e| (e, lock_file_path.clone()))?;
    Ok(())
}

fn lock_file_path(path: &Path, lock_id: Uuid) -> String {
    format!("{}/{}.lock", path.to_string_lossy(), lock_id)
}

fn locks_glob(path: &Path) -> String {
    format!("{}/*.lock", path.to_string_lossy())
}

fn wait_for_only_my_locks_left(lock_id: Uuid, index_directory: &Path) -> Result<bool, BakareError> {
    let my_lock_file_path = lock_file_path(index_directory, lock_id);

    let mut tries_left = 16;
    while tries_left > 0 {
        let mut locks = glob(&locks_glob(index_directory))?;
        let only_my_locks = locks.all(|path| match path {
            Ok(path) => path.to_string_lossy() == my_lock_file_path,
            Err(_) => false,
        });
        if only_my_locks {
            return Ok(true);
        }
        let wait_time = u64::from(rand::random::<u8>());
        thread::sleep(Duration::from_millis(wait_time));
        tries_left -= 1;
    }
    Ok(false)
}

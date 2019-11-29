use std::io::Write;
use std::path::Path;
use std::time::Duration;
use std::{fs, thread};

use atomicwrites::AtomicFile;
use atomicwrites::OverwriteBehavior::DisallowOverwrite;
use uuid::Uuid;

use glob::{glob, Paths};

use anyhow::Result;

pub fn release_lock(path: &Path, lock_id: Uuid) -> Result<()> {
    let lock_file_path = lock_file_path(path, lock_id);
    delete_lock_file(lock_file_path)?;
    Ok(())
}

fn delete_lock_file(lock_file_path: String) -> Result<()> {
    fs::remove_file(lock_file_path.clone())?;
    Ok(())
}

pub fn acquire_lock(lock_id: Uuid, index_directory: &Path) -> Result<()> {
    wait_to_have_sole_lock(lock_id, index_directory)?;
    create_lock_file(lock_id, index_directory)?;
    Ok(())
}

pub fn wait_to_have_sole_lock(lock_id: Uuid, index_directory: &Path) -> Result<()> {
    while !sole_lock(lock_id, index_directory)? {
        thread::sleep(Duration::from_millis(u64::from(rand::random::<u8>())))
    }
    Ok(())
}

pub fn sole_lock(lock_id: Uuid, index_directory: &Path) -> Result<bool> {
    let my_lock_file_path = lock_file_path(index_directory, lock_id);
    let mut locks = all_locks(index_directory)?;
    let only_my_locks = locks.all(|path| match path {
        Ok(path) => path.to_string_lossy() == my_lock_file_path,
        Err(_) => false,
    });
    Ok(only_my_locks)
}

fn all_locks(index_directory: &Path) -> Result<Paths> {
    Ok(glob(&locks_glob(index_directory))?)
}

fn create_lock_file(lock_id: Uuid, index_directory: &Path) -> Result<()> {
    let lock_file_path = lock_file_path(index_directory, lock_id);
    let file = AtomicFile::new(&lock_file_path, DisallowOverwrite);
    file.write(|f| f.write_all(lock_id.as_bytes()))?;
    Ok(())
}

fn lock_file_path(path: &Path, lock_id: Uuid) -> String {
    format!("{}/{}.lock", path.to_string_lossy(), lock_id)
}

fn locks_glob(path: &Path) -> String {
    format!("{}/*.lock", path.to_string_lossy())
}

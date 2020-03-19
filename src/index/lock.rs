use async_log::span;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, thread};

use uuid::Uuid;

use glob::{glob, Paths};

use anyhow::Context;
use anyhow::Result;
use std::fs::File;

pub fn release_lock(path: &Path, lock_id: Uuid) -> Result<()> {
    let lock_file_path = lock_file_path(path, lock_id);
    delete_lock_file(lock_file_path)?;
    Ok(())
}

fn delete_lock_file(lock_file_path: PathBuf) -> Result<()> {
    if lock_file_path.exists() {
        fs::remove_file(lock_file_path)?;
    }
    Ok(())
}

pub fn acquire_lock(lock_id: Uuid, index_directory: &Path) -> Result<()> {
    wait_to_have_sole_lock(lock_id, index_directory)?;
    Ok(())
}

pub fn wait_to_have_sole_lock(lock_id: Uuid, index_directory: &Path) -> Result<()> {
    span!("waiting for sole lock to be {}", lock_id, {
        while !sole_lock(lock_id, index_directory)? {
            release_lock(index_directory, lock_id)?;
            thread::sleep(Duration::from_millis(u64::from(rand::random::<u8>())));
            create_lock_file(lock_id, index_directory)?;
        }
    });
    Ok(())
}

pub fn sole_lock(lock_id: Uuid, index_directory: &Path) -> Result<bool> {
    let my_lock_file_path = lock_file_path(index_directory, lock_id);
    let mut locks = all_locks(index_directory)?;
    let only_my_locks = locks.all(|path| match path {
        Ok(path) => path.to_string_lossy() == my_lock_file_path.to_string_lossy(),
        Err(_) => false,
    });
    Ok(only_my_locks)
}

fn all_locks(index_directory: &Path) -> Result<Paths> {
    Ok(glob(&locks_glob(index_directory))?)
}

fn create_lock_file(lock_id: Uuid, index_directory: &Path) -> Result<()> {
    let lock_file_path = lock_file_path(index_directory, lock_id);
    let mut file = File::create(&lock_file_path).context("creating lock file")?;
    file.write_all(lock_id.as_bytes()).context("create lock file")?;
    Ok(())
}

fn lock_file_path(path: &Path, lock_id: Uuid) -> PathBuf {
    let path_text = &format!("{}/{}.lock", path.to_string_lossy(), lock_id);
    Path::new(path_text).to_path_buf()
}

fn locks_glob(path: &Path) -> String {
    format!("{}/*.lock", path.to_string_lossy())
}

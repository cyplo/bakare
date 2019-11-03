use std::path::Path;
use std::{fs, thread};

use rayon::prelude::*;
use tempfile::tempdir;

use bakare::error::BakareError;
use bakare::repository::Repository;
use bakare::source::TempSource;
use bakare::test::assertions::*;
use bakare::{backup, restore};
use std::time::Duration;

#[test]
fn handle_concurrent_backups() -> Result<(), BakareError> {
    let repository_path = &tempdir().unwrap().into_path();
    Repository::init(repository_path)?;

    let parallel_backups_number = 1;
    let files_per_backup_number = 16;
    let total_number_of_files = parallel_backups_number * files_per_backup_number;
    let finished_backup_runs = backup_in_parallel(repository_path, parallel_backups_number, files_per_backup_number)?;
    assert_eq!(finished_backup_runs.len(), parallel_backups_number);

    let all_restored_files = restore_all(repository_path)?;
    assert_eq!(all_restored_files.len(), total_number_of_files);

    for i in 0..parallel_backups_number {
        for j in 0..files_per_backup_number {
            let id = file_id(i, j);
            let file = all_restored_files.iter().find(|f| f.ends_with(id.clone()));
            assert!(file.unwrap().exists(), "file {:?} does not exist", file);
            let contents = fs::read_to_string(file.unwrap()).unwrap();
            assert_eq!(id.to_string(), contents.to_owned());
        }
    }
    Ok(())
}

fn file_id(i: usize, j: usize) -> String {
    format!("{}-{}", i, j)
}

fn backup_in_parallel<T>(
    repository_path: T,
    parallel_backups_number: usize,
    files_per_backup_number: usize,
) -> Result<Vec<usize>, BakareError>
where
    T: AsRef<Path> + Sync,
{
    (0..parallel_backups_number)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|task_number| {
            thread::sleep(Duration::from_millis(u64::from(rand::random::<u8>())));
            let mut repository = Repository::open(repository_path.as_ref())?;
            let source = TempSource::new().unwrap();
            let mut backup_engine = backup::Engine::new(source.path(), &mut repository)?;
            for i in 0..files_per_backup_number {
                let id = file_id(*task_number, i);
                source.write_text_to_file(&id, &id).unwrap();
            }
            backup_engine.backup()?;
            Ok(*task_number)
        })
        .collect()
}

fn restore_all<T: AsRef<Path>>(repository_path: T) -> Result<Vec<Box<Path>>, BakareError> {
    let restore_target = tempdir().unwrap().into_path();
    let mut restore_repository = Repository::open(repository_path.as_ref())?;
    let side_indexes_path = repository_path.as_ref().join("side_indexes");
    let side_indexes = get_sorted_files_recursively(side_indexes_path)?;
    assert_eq!(side_indexes.iter().count(), 0);
    let mut restore_engine = restore::Engine::new(&mut restore_repository, restore_target.as_ref())?;
    restore_engine.restore_all()?;
    get_sorted_files_recursively(&restore_target)
}

// TODO handle stale leftover locks

use std::fs;
use std::path::Path;

use anyhow::Result;
use async_log::span;
use bakare::repository::Repository;
use bakare::source::TempSource;
use bakare::test::assertions::*;
use bakare::{backup, restore};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{fork, getpid, ForkResult};
use tempfile::tempdir;

#[test]
fn handle_concurrent_backups() -> Result<()> {
    setup_logger();
    let repository_path = &tempdir().unwrap().into_path();
    Repository::init(repository_path)?;

    let parallel_backups_number = 16;
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

fn backup_in_parallel<T>(
    repository_path: T,
    parallel_backups_number: usize,
    files_per_backup_number: usize,
) -> Result<Vec<usize>>
where
    T: AsRef<Path> + Sync,
{
    let task_numbers = (0..parallel_backups_number).collect::<Vec<_>>();
    let mut child_pids = vec![];
    span!("[{}] acquiring children for parent", getpid(), {
        for task_number in &task_numbers {
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => {
                    child_pids.push(child);
                }
                Ok(ForkResult::Child) => {
                    backup_process(*task_number, &repository_path, files_per_backup_number)?;
                    std::process::exit(0);
                }

                Err(_) => panic!("fork failed"),
            }
        }
    });
    span!("[{}] waiting for {} children", getpid(), child_pids.len(), {
        for pid in child_pids {
            log::debug!("[{}] waiting for a child {} to exit", getpid(), pid);
            let status = waitpid(Some(pid), None)?;
            match status {
                WaitStatus::Exited(pid, code) => {
                    assert!(code == 0, "failed the wait for {} with code {}", pid, code);
                }
                WaitStatus::Signaled(pid, _, _) => assert!(false, "failed with signal for {}", pid),
                _ => panic!("unknown state"),
            }
        }
    });
    Ok(task_numbers)
}

fn backup_process<T>(task_number: usize, repository_path: T, files_per_backup_number: usize) -> Result<()>
where
    T: AsRef<Path> + Sync,
{
    let mut repository = Repository::open(repository_path.as_ref())?;
    let source = TempSource::new().unwrap();
    let mut backup_engine = backup::Engine::new(source.path(), &mut repository)?;
    for i in 0..files_per_backup_number {
        let id = file_id(task_number, i);
        source.write_text_to_file(&id, &id).unwrap();
    }
    backup_engine.backup()?;
    Ok(())
}

fn restore_all<T: AsRef<Path>>(repository_path: T) -> Result<Vec<Box<Path>>> {
    let restore_target = tempdir().unwrap().into_path();
    let mut restore_repository = Repository::open(repository_path.as_ref())?;
    let mut restore_engine = restore::Engine::new(&mut restore_repository, restore_target.as_ref())?;
    restore_engine.restore_all()?;
    get_sorted_files_recursively(&restore_target)
}

fn setup_logger() {
    let logger = femme::pretty::Logger::new();
    async_log::Logger::wrap(logger, rand::random::<u64>)
        .start(log::LevelFilter::Trace)
        .unwrap();
}

fn file_id(i: usize, j: usize) -> String {
    format!("{}-{}", i, j)
}
// TODO handle stale leftover locks

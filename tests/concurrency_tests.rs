#[cfg(test)]
mod must {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use anyhow::Result;
    use bakare::test::source::TestSource;
    use bakare::{backup, restore};
    use bakare::{repository::Repository, test::assertions::in_memory::*};
    use nix::unistd::{fork, ForkResult};
    use nix::{
        sys::wait::{waitpid, WaitStatus},
        unistd::getpid,
    };
    use tempfile::tempdir;

    #[test]
    fn handle_concurrent_backups() -> Result<()> {
        setup_logger();

        let dir = tempdir()?;
        let repository_path = dir.path();
        let repository_path = repository_path.join(&format!("repository-{}", getpid()));
        Repository::init(&repository_path)?;

        let parallel_backups_number = 16;
        let files_per_backup_number = 16;
        let total_number_of_files = parallel_backups_number * files_per_backup_number;

        let finished_backup_runs = backup_in_parallel(&repository_path, parallel_backups_number, files_per_backup_number)?;
        assert_eq!(finished_backup_runs.len(), parallel_backups_number);
        assert!(data_weight(&repository_path)? > 0);

        let target_path = tempdir()?;
        let all_restored_files = restore_all(&repository_path, target_path.path())?;
        assert_eq!(all_restored_files.len(), total_number_of_files);

        assert_all_files_in_place(parallel_backups_number, files_per_backup_number, &all_restored_files)?;
        Ok(())
    }

    fn assert_all_files_in_place(
        parallel_backups_number: usize,
        files_per_backup_number: usize,
        all_restored_files: &[PathBuf],
    ) -> Result<()> {
        for i in 0..parallel_backups_number {
            for j in 0..files_per_backup_number {
                let id = file_id(i, j);
                let file = all_restored_files
                    .iter()
                    .find(|f| f.file_name().unwrap().to_string_lossy() == id);
                assert!(file.unwrap().exists(), "file {:?} does not exist", file);
                let contents = fs::read_to_string(file.unwrap())?;
                assert_eq!(id.to_string(), contents.to_owned());
            }
        }
        Ok(())
    }

    fn backup_in_parallel(
        repository_path: &Path,
        parallel_backups_number: usize,
        files_per_backup_number: usize,
    ) -> Result<Vec<usize>> {
        let task_numbers = (0..parallel_backups_number).collect::<Vec<_>>();
        let mut child_pids = vec![];
        for task_number in &task_numbers {
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => {
                    child_pids.push(child);
                }
                Ok(ForkResult::Child) => {
                    backup_process(*task_number, repository_path, files_per_backup_number)?;
                    std::process::exit(0);
                }

                Err(_) => panic!("fork failed"),
            }
        }
        for pid in child_pids {
            let status = waitpid(Some(pid), None)?;
            match status {
                WaitStatus::Exited(pid, code) => {
                    assert!(code == 0, "failed the wait for {} with code {}", pid, code);
                }
                WaitStatus::Signaled(pid, _, _) => panic!("failed with signal for {}", pid),
                _ => panic!("unknown state"),
            }
        }
        Ok(task_numbers)
    }

    fn backup_process(task_number: usize, repository_path: &Path, files_per_backup_number: usize) -> Result<()> {
        let mut repository = Repository::open(repository_path)?;
        let source = TestSource::new().unwrap();
        let mut backup_engine = backup::Engine::new(source.path(), &mut repository)?;
        for i in 0..files_per_backup_number {
            let id = file_id(task_number, i);
            source.write_text_to_file(&id, &id).unwrap();
        }
        backup_engine.backup()?;
        Ok(())
    }

    fn restore_all(repository_path: &Path, restore_target: &Path) -> Result<Vec<PathBuf>> {
        let mut restore_repository = Repository::open(repository_path)?;
        let mut restore_engine = restore::Engine::new(&mut restore_repository, restore_target)?;
        restore_engine.restore_all()?;
        get_sorted_files_recursively(restore_target)
    }

    fn setup_logger() {
        femme::with_level(log::LevelFilter::Info);
    }

    fn file_id(i: usize, j: usize) -> String {
        format!("{}-{}", i, j)
    }
}

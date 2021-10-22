pub mod in_memory {
    use std::{
        fs::File,
        io::Read,
        path::{Path, PathBuf},
    };

    use crate::{
        backup,
        repository::{item::RepositoryItem, ItemId, Repository},
        restore,
        test::source::TestSource,
    };
    use anyhow::Result;

    use tempfile::tempdir;
    use walkdir::WalkDir;

    pub fn assert_same_after_restore(source_path: &Path) -> Result<()> {
        let repository_path = tempdir()?;
        let restore_target = tempdir()?;

        Repository::init(repository_path.path())?;
        {
            let mut backup_repository = Repository::open(repository_path.path())?;
            let mut backup_engine = backup::Engine::new(source_path, &mut backup_repository)?;
            backup_engine.backup()?;
        }
        {
            let mut restore_repository = Repository::open(repository_path.path())?;

            let mut restore_engine = restore::Engine::new(&mut restore_repository, restore_target.path())?;
            restore_engine.restore_all()?;
        }

        assert_directory_trees_have_same_contents(source_path, restore_target.path())?;
        Ok(())
    }

    pub fn assert_restored_file_contents(repository_path: &Path, source_file_full_path: &Path, contents: &[u8]) -> Result<()> {
        let mut restore_repository = Repository::open(repository_path)?;
        let item = restore_repository.newest_item_by_source_path(source_file_full_path)?;
        let restore_target = tempdir()?;
        let restore_engine = restore::Engine::new(&mut restore_repository, restore_target.path())?;

        restore_engine.restore(&item.unwrap())?;
        let source_file_relative_path = Path::new(source_file_full_path).strip_prefix("/")?;
        let restored_file_path = restore_target.path().join(&source_file_relative_path);
        assert_target_file_contents(&restored_file_path, contents)
    }

    pub fn assert_restored_from_version_has_contents(
        repository_path: &Path,
        source_file_full_path: &Path,
        old_contents: &[u8],
        old_id: &ItemId,
    ) -> Result<()> {
        let mut restore_repository = Repository::open(repository_path)?;
        let old_item = restore_repository.item_by_id(old_id)?;
        let restore_target = tempdir()?;
        let restore_engine = restore::Engine::new(&mut restore_repository, restore_target.path())?;
        restore_engine.restore(&old_item.unwrap())?;
        let source_file_relative_path = Path::new(source_file_full_path).strip_prefix("/")?;
        let restored_file_path = restore_target.path().join(&source_file_relative_path);
        assert_target_file_contents(&restored_file_path, old_contents)
    }

    pub fn newest_item(repository_path: &Path, source_file_full_path: &Path) -> Result<RepositoryItem> {
        let item = {
            let reading_repository = Repository::open(repository_path)?;
            let item = reading_repository.newest_item_by_source_path(source_file_full_path)?;
            assert!(item.is_some());
            item.unwrap()
        };
        Ok(item)
    }

    pub fn restore_all_from_reloaded_repository(repository_path: &Path, restore_target: &Path) -> Result<()> {
        {
            let mut restore_repository = Repository::open(repository_path)?;
            let mut restore_engine = restore::Engine::new(&mut restore_repository, restore_target)?;
            restore_engine.restore_all()?;
            Ok(())
        }
    }

    pub fn backup_file_with_text_contents(
        source: &TestSource,
        repository_path: &Path,
        source_file_relative_path: &str,
        contents: &str,
    ) -> Result<()> {
        {
            backup_file_with_byte_contents(source, repository_path, source_file_relative_path, contents.as_bytes())
        }
    }

    pub fn backup_file_with_byte_contents(
        source: &TestSource,
        repository_path: &Path,
        source_file_relative_path: &str,
        contents: &[u8],
    ) -> Result<()> {
        {
            let mut backup_repository = Repository::open(repository_path)?;

            let mut backup_engine = backup::Engine::new(source.path(), &mut backup_repository)?;
            source.write_bytes_to_file(source_file_relative_path, contents).unwrap();
            backup_engine.backup()?;
            Ok(())
        }
    }

    pub fn data_weight(repository_path: &Path) -> Result<u64> {
        {
            let repository = Repository::open(repository_path)?;
            repository.data_weight()
        }
    }

    fn assert_directory_trees_have_same_contents(left: &Path, right: &Path) -> Result<()> {
        let left_files = get_sorted_files_recursively(left)?;
        let right_files = get_sorted_files_recursively(right)?;

        let pairs = left_files.iter().zip(right_files);
        for (l, r) in pairs {
            assert_eq!(l.file_name(), r.file_name());
            let mut fl = File::open(l)?;
            let mut fr = File::open(r)?;
            let mut bl = vec![];
            let mut br = vec![];
            fl.read_to_end(&mut bl).unwrap();
            fr.read_to_end(&mut br).unwrap();
            assert_eq!(bl, br);
        }
        Ok(())
    }

    pub fn get_sorted_files_recursively(path: &Path) -> Result<Vec<PathBuf>> {
        assert!(
            path.exists(),
            "[get_sorted_files_recursively] invoked on a path that does not exist: {:?}",
            path
        );
        let walker = WalkDir::new(path);
        let result = walker
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.metadata().map_or(false, |m| m.is_file()))
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>();
        Ok(result)
    }

    fn assert_target_file_contents(restored_path: &Path, expected_contents: &[u8]) -> Result<()> {
        let mut actual_contents = vec![];
        assert!(
            restored_path.exists(),
            "expected '{}' to be there",
            restored_path.to_string_lossy()
        );
        let mut file = File::open(restored_path)?;
        file.read_to_end(&mut actual_contents)?;
        assert_eq!(expected_contents, actual_contents);
        Ok(())
    }
}

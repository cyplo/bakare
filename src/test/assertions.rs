pub mod in_memory {
    use std::path::Path;

    use crate::{
        backup,
        repository::{item::RepositoryItem, ItemId, Repository},
        restore,
        test::source::TestSource,
    };
    use anyhow::Result;
    use vfs::{MemoryFS, VfsFileType, VfsPath};

    use rand::Rng;

    pub fn random_in_memory_path(prefix: &str) -> Result<VfsPath> {
        let path: VfsPath = MemoryFS::new().into();
        let path = path.join(&format!("{}-{}", prefix, rand::thread_rng().gen::<u64>()))?;
        Ok(path)
    }

    pub fn assert_same_after_restore(source_path: &VfsPath) -> Result<()> {
        let repository_path: VfsPath = random_in_memory_path("repository")?;
        let restore_target: VfsPath = random_in_memory_path("target")?;

        assert_ne!(source_path, &repository_path);
        assert_ne!(repository_path, restore_target);

        Repository::init(&repository_path)?;
        {
            let mut backup_repository = Repository::open(&repository_path)?;
            let mut backup_engine = backup::Engine::new(source_path, &mut backup_repository)?;
            backup_engine.backup()?;
        }
        {
            let mut restore_repository = Repository::open(&repository_path)?;

            let mut restore_engine = restore::Engine::new(&mut restore_repository, &restore_target)?;
            restore_engine.restore_all()?;
        }

        assert_directory_trees_have_same_contents(source_path, &restore_target)?;
        Ok(())
    }

    pub fn assert_restored_file_contents(
        repository_path: &VfsPath,
        source_file_full_path: &VfsPath,
        contents: &[u8],
    ) -> Result<()> {
        let mut restore_repository = Repository::open(repository_path)?;
        let item = restore_repository.newest_item_by_source_path(&source_file_full_path)?;
        let restore_target = random_in_memory_path("target")?;
        let restore_engine = restore::Engine::new(&mut restore_repository, &restore_target)?;

        restore_engine.restore(&item.unwrap())?;
        let source_file_relative_path = Path::new(source_file_full_path.as_str()).strip_prefix("/")?;
        let restored_file_path = restore_target.join(&source_file_relative_path.to_string_lossy())?;
        assert_target_file_contents(&restored_file_path, contents)
    }

    pub fn assert_restored_from_version_has_contents(
        repository_path: &VfsPath,
        source_file_full_path: &VfsPath,
        old_contents: &[u8],
        old_id: &ItemId,
    ) -> Result<()> {
        let mut restore_repository = Repository::open(repository_path)?;
        let old_item = restore_repository.item_by_id(&old_id)?;
        let restore_target = random_in_memory_path("target")?;
        let restore_engine = restore::Engine::new(&mut restore_repository, &restore_target)?;
        restore_engine.restore(&old_item.unwrap())?;
        let source_file_relative_path = Path::new(source_file_full_path.as_str()).strip_prefix("/")?;
        let restored_file_path = restore_target.join(&source_file_relative_path.to_string_lossy())?;
        assert_target_file_contents(&restored_file_path, old_contents)
    }

    pub fn newest_item(repository_path: &VfsPath, source_file_full_path: &VfsPath) -> Result<RepositoryItem> {
        let item = {
            let reading_repository = Repository::open(repository_path)?;
            let item = reading_repository.newest_item_by_source_path(&source_file_full_path)?;
            assert!(item.is_some());
            item.unwrap()
        };
        Ok(item)
    }

    pub fn restore_all_from_reloaded_repository(repository_path: &VfsPath, restore_target: &VfsPath) -> Result<()> {
        {
            let mut restore_repository = Repository::open(repository_path)?;
            let mut restore_engine = restore::Engine::new(&mut restore_repository, &restore_target)?;
            restore_engine.restore_all()?;
            Ok(())
        }
    }

    pub fn backup_file_with_text_contents(
        source: &TestSource,
        repository_path: &VfsPath,
        source_file_relative_path: &str,
        contents: &str,
    ) -> Result<()> {
        {
            backup_file_with_byte_contents(source, repository_path, source_file_relative_path, contents.as_bytes())
        }
    }

    pub fn backup_file_with_byte_contents(
        source: &TestSource,
        repository_path: &VfsPath,
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

    pub fn data_weight(repository_path: &VfsPath) -> Result<u64> {
        {
            let repository = Repository::open(repository_path)?;
            Ok(repository.data_weight()?)
        }
    }

    fn assert_directory_trees_have_same_contents(left: &VfsPath, right: &VfsPath) -> Result<()> {
        let left_files = get_sorted_files_recursively(left)?;
        let right_files = get_sorted_files_recursively(right)?;

        let pairs = left_files.iter().zip(right_files);
        for (l, r) in pairs {
            assert_eq!(l.filename(), r.filename());
            let mut fl = l.open_file()?;
            let mut fr = r.open_file()?;
            let mut bl = vec![];
            let mut br = vec![];
            fl.read_to_end(&mut bl).unwrap();
            fr.read_to_end(&mut br).unwrap();
            assert_eq!(bl, br);
        }
        Ok(())
    }

    pub fn get_sorted_files_recursively(path: &VfsPath) -> Result<Vec<VfsPath>> {
        assert!(
            path.exists()?,
            "[get_sorted_files_recursively] invoked on a path that does not exist: {:?}",
            path
        );
        let walker = path.walk_dir()?;

        let mut result = vec![];

        for maybe_entry in walker {
            let entry = &maybe_entry?;
            if entry == path {
                continue;
            }
            if entry.metadata()?.file_type == VfsFileType::File {
                result.push(entry.clone());
            }
        }

        result.sort_by_key(|a| a.filename());

        Ok(result)
    }

    fn assert_target_file_contents(restored_path: &VfsPath, expected_contents: &[u8]) -> Result<()> {
        let mut actual_contents = vec![];
        assert!(restored_path.exists()?, "expected '{}' to be there", restored_path.as_str());
        restored_path.open_file()?.read_to_end(&mut actual_contents)?;
        assert_eq!(expected_contents, actual_contents);
        Ok(())
    }
}

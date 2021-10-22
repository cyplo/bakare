#[cfg(test)]
mod must {
    use anyhow::Result;
    use bakare::backup;
    use bakare::test::assertions::in_memory::*;
    use bakare::{repository::Repository, test::source::TestSource};
    use tempfile::tempdir;

    #[test]
    fn restore_multiple_files() -> Result<()> {
        let source = TestSource::new().unwrap();

        source.write_text_to_file("first", "some contents").unwrap();
        source.write_text_to_file("second", "some contents").unwrap();
        source.write_text_to_file("third", "some other contents").unwrap();

        assert_same_after_restore(source.path())
    }

    #[test]
    fn restore_files_after_reopening_repository() -> Result<()> {
        let source = TestSource::new()?;
        let dir = tempdir()?;
        let repository_path = dir.path();
        let restore_target = tempdir()?;

        Repository::init(repository_path)?;

        let source_file_relative_path = "some file path";
        let original_contents = "some old contents";

        backup_file_with_text_contents(&source, repository_path, source_file_relative_path, original_contents)?;

        restore_all_from_reloaded_repository(repository_path, restore_target.path())?;

        let source_file_full_path = &source.file_path(source_file_relative_path)?;
        assert_restored_file_contents(repository_path, source_file_full_path, original_contents.as_bytes())
    }

    #[test]
    fn restore_older_version_of_file() -> Result<()> {
        let source = TestSource::new().unwrap();
        let dir = tempdir()?;
        let repository_path = dir.path();
        Repository::init(repository_path)?;

        let source_file_relative_path = "some path";
        let source_file_full_path = source.file_path(source_file_relative_path)?;
        let old_contents = "some old contents";

        backup_file_with_text_contents(&source, repository_path, source_file_relative_path, old_contents)?;

        let old_item = newest_item(repository_path, &source_file_full_path)?;
        let old_id = old_item.id();

        let new_contents = "totally new contents";
        backup_file_with_text_contents(&source, repository_path, source_file_relative_path, new_contents)?;

        assert_restored_from_version_has_contents(repository_path, &source_file_full_path, old_contents.as_bytes(), old_id)
    }

    #[test]
    fn newer_version_should_be_greater_than_earlier_version() -> Result<()> {
        let source = TestSource::new().unwrap();
        let dir = tempdir()?;
        let repository_path = dir.path();
        Repository::init(repository_path)?;

        let source_file_relative_path = "some path";
        let source_file_full_path = source.file_path(source_file_relative_path)?;

        backup_file_with_text_contents(&source, repository_path, source_file_relative_path, "old")?;

        let old_item = newest_item(repository_path, &source_file_full_path)?;
        let old_version = old_item.version();

        backup_file_with_text_contents(&source, repository_path, source_file_relative_path, "new")?;

        let new_item = newest_item(repository_path, &source_file_full_path)?;
        let new_version = new_item.version();

        assert!(new_version > old_version);

        Ok(())
    }

    #[test]
    fn restore_latest_version_by_default() -> Result<()> {
        let source = TestSource::new().unwrap();
        let dir = tempdir()?;
        let repository_path = dir.path();
        Repository::init(repository_path)?;

        let source_file_relative_path = "some path";
        backup_file_with_text_contents(&source, repository_path, source_file_relative_path, "old contents")?;
        backup_file_with_text_contents(&source, repository_path, source_file_relative_path, "newer contents")?;
        backup_file_with_text_contents(&source, repository_path, source_file_relative_path, "newest contents")?;

        let source_file_full_path = &source.file_path(source_file_relative_path)?;
        assert_restored_file_contents(repository_path, source_file_full_path, b"newest contents")
    }

    #[test]
    fn forbid_backup_of_paths_within_repository() -> Result<()> {
        let dir = tempdir()?;
        let repository_path = dir.path();
        Repository::init(repository_path)?;
        let mut repository = Repository::open(repository_path)?;

        let error = backup::Engine::new(repository_path, &mut repository);
        assert!(error.is_err());
        Ok(())
    }
    // TODO: index corruption
    // TODO: encryption
    // TODO: resume from sleep while backup in progress
}

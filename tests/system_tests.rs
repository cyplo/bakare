#[cfg(test)]
mod must {
    use std::fs::File;
    use std::io::Read;

    use bakare::backup;
    use bakare::test::assertions::in_memory::*;
    use bakare::{repository::Repository, test::source::TestSource};

    use anyhow::Result;
    use proptest::prelude::*;
    use tempfile::tempdir;
    use walkdir::WalkDir;

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
        let secret = "some secret";
        Repository::init(repository_path, secret)?;

        let source_file_relative_path = "some file path";
        let original_contents = "some old contents";

        backup_file_with_text_contents(&source, repository_path, secret, source_file_relative_path, original_contents)?;

        restore_all_from_reloaded_repository(repository_path, secret, restore_target.path())?;

        let source_file_full_path = &source.file_path(source_file_relative_path)?;
        assert_restored_file_contents(repository_path, secret, source_file_full_path, original_contents.as_bytes())
    }

    #[test]
    fn restore_older_version_of_file() -> Result<()> {
        let source = TestSource::new().unwrap();
        let dir = tempdir()?;
        let repository_path = dir.path();
        let secret = "some secret";
        Repository::init(repository_path, secret)?;

        let source_file_relative_path = "some path";
        let source_file_full_path = source.file_path(source_file_relative_path)?;
        let old_contents = "some old contents";

        backup_file_with_text_contents(&source, repository_path, secret, source_file_relative_path, old_contents)?;

        let old_item = newest_item(repository_path, secret, &source_file_full_path)?;
        let old_id = old_item.id();

        let new_contents = "totally new contents";
        backup_file_with_text_contents(&source, repository_path, secret, source_file_relative_path, new_contents)?;

        assert_restored_from_version_has_contents(
            repository_path,
            secret,
            &source_file_full_path,
            old_contents.as_bytes(),
            old_id,
        )
    }

    #[test]
    fn newer_version_should_be_greater_than_earlier_version() -> Result<()> {
        let source = TestSource::new().unwrap();
        let dir = tempdir()?;
        let repository_path = dir.path();
        let secret = "some secret";
        Repository::init(repository_path, secret)?;

        let source_file_relative_path = "some path";
        let source_file_full_path = source.file_path(source_file_relative_path)?;

        backup_file_with_text_contents(&source, repository_path, secret, source_file_relative_path, "old")?;

        let old_item = newest_item(repository_path, secret, &source_file_full_path)?;
        let old_version = old_item.version();

        backup_file_with_text_contents(&source, repository_path, secret, source_file_relative_path, "new")?;

        let new_item = newest_item(repository_path, secret, &source_file_full_path)?;
        let new_version = new_item.version();

        assert!(new_version > old_version);

        Ok(())
    }

    #[test]
    fn restore_latest_version_by_default() -> Result<()> {
        let source = TestSource::new().unwrap();
        let dir = tempdir()?;
        let repository_path = dir.path();
        let secret = "some secret";
        Repository::init(repository_path, secret)?;

        let source_file_relative_path = "some path";
        backup_file_with_text_contents(&source, repository_path, secret, source_file_relative_path, "old contents")?;
        backup_file_with_text_contents(&source, repository_path, secret, source_file_relative_path, "newer contents")?;
        backup_file_with_text_contents(&source, repository_path, secret, source_file_relative_path, "newest contents")?;

        let source_file_full_path = &source.file_path(source_file_relative_path)?;
        assert_restored_file_contents(repository_path, secret, source_file_full_path, b"newest contents")
    }

    #[test]
    fn forbid_backup_of_paths_within_repository() -> Result<()> {
        let dir = tempdir()?;
        let repository_path = dir.path();
        let secret = "some secret";
        Repository::init(repository_path, secret)?;
        let mut repository = Repository::open(repository_path, secret)?;

        let error = backup::Engine::new(repository_path, &mut repository);
        assert!(error.is_err());
        Ok(())
    }

    proptest! {
        #[test]
        fn allow_searching_by_filename(filename in "[a-zA-Z]{3,}"){
            let source = TestSource::new().unwrap();
            let dir = tempdir()?;
            let repository_path = dir.path();
        let secret = "some secret";
            Repository::init(repository_path, secret).unwrap();

            backup_file_with_text_contents(&source, repository_path, secret, &filename, "some contents").unwrap();

            let repository = Repository::open(repository_path, secret).unwrap();

            let second_file = repository.find_latest_by_path_fragment(&filename).unwrap().unwrap();
            assert_eq!(second_file.original_source_path(), source.file_path(&filename).unwrap().as_os_str());
        }

        #[test]
        fn not_leak_file_names_via_file_names(filename in "[a-zA-Z]{4,}"){
            let source = TestSource::new().unwrap();
            let dir = tempdir()?;
            let repository_path = dir.path();
        let secret = "some secret";
            Repository::init(repository_path, secret).unwrap();

            backup_file_with_text_contents(&source, repository_path, secret, &filename, "some contents").unwrap();

            let walker = WalkDir::new(repository_path);
            let matching_paths = walker
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().as_os_str().to_string_lossy().contains(&filename));

            assert_eq!(matching_paths.count(), 0);
        }

        #[test]
        fn not_leak_file_names_via_file_contents(test_filename in "[a-zA-Z]{8,}"){
            let source = TestSource::new().unwrap();
            let dir = tempdir()?;
            let repository_path = dir.path();
        let secret = "some secret";
            Repository::init(repository_path, secret).unwrap();

            backup_file_with_text_contents(&source, repository_path, secret, &test_filename, "some contents").unwrap();

            let all_repo_files = get_sorted_files_recursively(repository_path).unwrap();
            assert!(!all_repo_files.is_empty());

            for filepath in all_repo_files {
                let mut file = File::open(&filepath).unwrap();
                let filename = &filepath.as_os_str().to_string_lossy();
                let mut contents = vec![];
                file.read_to_end(&mut contents).unwrap();
                let test_filename_bytes = test_filename.as_bytes();
                let contains = contents.windows(test_filename_bytes.len()).any(move |sub_slice| sub_slice == test_filename_bytes);
                assert!(!contains, "file {} in the repository directory contains plain text file name '{}' that was previously backed up", filename, test_filename);
            }
        }
    }
    // TODO: encryption
    // TODO: resume from sleep while backup in progress
}

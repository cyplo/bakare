use bakare::backup;
use bakare::restore;

use bakare::error::BakareError;
use bakare::repository::Repository;
use bakare::source::TempSource;

use bakare::test::assertions::{assert_same_after_restore, assert_target_file_contents};
use std::fs;
use tempfile::tempdir;

#[test]
fn restore_multiple_files() -> Result<(), BakareError> {
    let source = TempSource::new()?;

    source.write_text_to_file("first", "some contents")?;
    source.write_text_to_file("second", "some contents")?;
    source.write_text_to_file("third", "some other contents")?;

    assert_same_after_restore(source.path())
}

#[test]
fn restore_files_after_reopening_repository() -> Result<(), BakareError> {
    let source = TempSource::new()?;
    let repository_path = tempdir()?.into_path();
    let restore_target = tempdir()?.into_path();
    Repository::init(repository_path.as_path())?;

    let source_file_relative_path = "some file path";
    let original_contents = "some old contents";

    {
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source.path(), &mut backup_repository);
        source.write_text_to_file(source_file_relative_path, original_contents)?;
        backup_engine.backup()?;
    }

    {
        let restore_repository = Repository::open(repository_path.as_path())?;
        let restore_engine = restore::Engine::new(&restore_repository, &restore_target)?;
        restore_engine.restore_all()?;
    }

    let source_file_full_path = source.file_path(source_file_relative_path);
    let restored_file_path = restore_target.join(source_file_full_path.strip_prefix("/")?);
    let contents = fs::read_to_string(restored_file_path)?;

    assert_eq!(contents, original_contents);
    Ok(())
}

#[test]
fn restore_older_version_of_file() -> Result<(), BakareError> {
    let source = TempSource::new()?;
    let repository_path = tempdir()?.into_path();
    Repository::init(repository_path.as_path())?;

    let source_file_relative_path = "some path";
    let source_file_full_path = source.file_path(source_file_relative_path);
    let old_contents = "some old contents";

    {
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source.path(), &mut backup_repository);
        source.write_text_to_file(source_file_relative_path, old_contents)?;
        backup_engine.backup()?;
    }

    let old_version = {
        let reading_repository = Repository::open(repository_path.as_path())?;
        let item = reading_repository.item_by_source_path(&source_file_full_path)?;
        assert!(item.is_some());
        let item = item.unwrap();
        item.version().clone()
    };

    {
        let new_contents = "totally new contents";
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source.path(), &mut backup_repository);
        source.write_text_to_file(source_file_relative_path, new_contents)?;
        backup_engine.backup()?;
    }

    let restore_repository = Repository::open(repository_path.as_path())?;
    let restore_target = tempdir()?;
    let restore_engine = restore::Engine::new(&restore_repository, &restore_target.path())?;
    let old_item = restore_repository.item_by_source_path_and_version(&source_file_full_path, &old_version)?;
    restore_engine.restore(&old_item.unwrap())?;

    let restored_file_path = restore_target.path().join(source_file_full_path.strip_prefix("/")?);
    assert_target_file_contents(&restored_file_path, old_contents)
}

// TODO: restore latest version by default
// TODO: deduplicate data
// TODO: test that index is stored separately from data
// TODO: index corruption
// TODO: forbid source within repository

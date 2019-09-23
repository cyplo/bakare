use tempfile::tempdir;

use bakare::backup;
use bakare::error::BakareError;
use bakare::repository::Repository;
use bakare::source::TempSource;
use bakare::test::assertions::*;

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
    let repository_path = &tempdir()?.into_path();
    let restore_target = tempdir()?.into_path();
    Repository::init(repository_path)?;

    let source_file_relative_path = "some file path";
    let original_contents = "some old contents";

    backup_file_with_contents(&source, &repository_path, source_file_relative_path, original_contents)?;

    restore_all_from_reloaded_repository(&repository_path, &restore_target)?;

    let source_file_full_path = &source.file_path(source_file_relative_path);
    assert_restored_has_contents(repository_path, source_file_full_path, "newest contents")
}

#[test]
fn restore_older_version_of_file() -> Result<(), BakareError> {
    let source = TempSource::new()?;
    let repository_path = tempdir()?.into_path();
    Repository::init(repository_path.as_path())?;

    let source_file_relative_path = "some path";
    let source_file_full_path = source.file_path(source_file_relative_path);
    let old_contents = "some old contents";

    backup_file_with_contents(&source, &repository_path, source_file_relative_path, old_contents)?;

    let old_version = item_id(&repository_path, &source_file_full_path)?;

    let new_contents = "totally new contents";
    backup_file_with_contents(&source, &repository_path, source_file_relative_path, new_contents)?;

    assert_restored_from_version_has_contents(&repository_path, &source_file_full_path, old_contents, &old_version)
}

#[test]
fn restore_latest_version_by_default() -> Result<(), BakareError> {
    let source = TempSource::new()?;
    let repository_path = &tempdir()?.into_path();
    Repository::init(repository_path)?;

    let source_file_relative_path = "some path";
    backup_file_with_contents(&source, &repository_path, source_file_relative_path, "old contents")?;
    backup_file_with_contents(&source, &repository_path, source_file_relative_path, "newer contents")?;
    backup_file_with_contents(&source, &repository_path, source_file_relative_path, "newest contents")?;

    let source_file_full_path = &source.file_path(source_file_relative_path);
    assert_restored_has_contents(repository_path, source_file_full_path, "newest contents")
}

#[test]
fn forbid_backup_of_paths_within_repository() -> Result<(), BakareError> {
    let repository_path = &tempdir()?.into_path();
    Repository::init(repository_path)?;
    let mut repository = Repository::open(repository_path)?;
    let error = backup::Engine::new(repository_path, &mut repository).err().unwrap();
    let correct_error = match error {
        BakareError::SourceSameAsRepository => true,
        _ => false,
    };
    assert!(correct_error);
    Ok(())
}

// TODO: deduplicate data
// TODO: test that index is stored separately from data
// TODO: index corruption
// TODO: newer version should be greater than older version
// TODO: split version into file id and version

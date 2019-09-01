use bakare::backup;
use bakare::restore;

use bakare::source::TempSource;

use bakare::error::BakareError;
use bakare::repository::Repository;
use dir_diff::is_different;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn restore_backed_up_files() -> Result<(), BakareError> {
    let source = TempSource::new()?;

    source.write_text_to_file("first", "some contents")?;
    source.write_text_to_file("second", "some contents")?;
    source.write_text_to_file("third", "some other contents")?;

    assert_same_after_restore(source.path())
}

#[test]
fn restore_older_version_of_file() -> Result<(), BakareError> {
    let source = TempSource::new()?;
    let repository_path = tempdir()?.into_path();
    let restore_repository = Repository::open(repository_path.as_path())?;

    let relative_path_text = "some path";
    let file_path = source.file_path(relative_path_text);
    let new_contents = "totally new contents";
    let restore_target = tempdir()?;
    let restore_engine = restore::Engine::new(&restore_repository, &restore_target.path());
    let old_contents = "some old contents";

    {
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source.path(), &mut backup_repository);
        source.write_text_to_file(relative_path_text, old_contents)?;
        backup_engine.backup()?;
    }

    let backup_repository = Repository::open(repository_path.as_path())?;
    let file_id = backup_repository.item(&file_path);
    assert!(file_id.is_some());
    let file_id = file_id.unwrap();
    let old_version = file_id.version();

    {
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source.path(), &mut backup_repository);
        source.write_text_to_file(relative_path_text, new_contents)?;
        backup_engine.backup()?;
    }

    restore_engine.restore_as_of_version(&file_id, old_version)?;

    assert_target_file_contents(restore_target.path(), relative_path_text, old_contents)
}

fn assert_target_file_contents(target: &Path, filename: &str, expected_contents: &str) -> Result<(), BakareError> {
    let restored_path = target.join(filename);
    let mut actual_contents = String::new();
    File::open(restored_path)?.read_to_string(&mut actual_contents)?;
    assert_eq!(expected_contents, actual_contents);
    Ok(())
}

fn assert_same_after_restore(source_path: &Path) -> Result<(), BakareError> {
    let repository_path = tempdir()?.into_path();
    let restore_target = tempdir()?;

    {
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source_path, &mut backup_repository);
        backup_engine.backup()?;
    }
    {
        let mut restore_repository = Repository::open(repository_path.as_path())?;
        let restore_engine = restore::Engine::new(&mut restore_repository, &restore_target.path());
        restore_engine.restore_all()?;
    }
    let are_source_and_target_different = is_different(source_path, &restore_target.path()).unwrap();
    assert!(!are_source_and_target_different);
    Ok(())
}

// TODO: restore latest version by default
// TODO: deduplicate data

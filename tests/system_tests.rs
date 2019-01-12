use bakare::backup;
use bakare::restore;

use bakare::source::Source;

use dir_diff::is_different;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tempfile::tempdir;
use bakare::error::BakareError;
use bakare::repository::Repository;

#[test]
fn restore_backed_up_files() -> Result<(), BakareError> {
    let source = Source::new()?;
    let repository_path = tempdir()?.into_path();
    let repository = Repository::new(repository_path.as_path())?;

    source.write_text_to_file("first", "some contents")?;
    source.write_text_to_file("second", "some contents")?;
    source.write_text_to_file("third", "some other contents")?;

    assert_same_after_restore(source.path(), &repository)
}

#[test]
fn restore_older_version_of_file() -> Result<(), BakareError> {
    let source = Source::new()?;
    let repository_path = tempdir()?.into_path();
    let repository = Repository::new(repository_path.as_path())?;
    let backup_engine = backup::Engine::new(source.path(), &repository);
    let file_path = "some path";
    let new_contents = "totally new contents";
    let restore_target = tempdir()?;
    let restore_engine = restore::Engine::new(&repository, &restore_target.path());
    let old_contents = "some old contents";

    source.write_text_to_file(file_path, old_contents)?;
    backup_engine.backup()?;
    let repository_path = repository.relative_path(file_path);
    let file_id = repository.file_id(&repository_path)?;
    let old_version = repository.newest_version_for(&file_id)?;

    source.write_text_to_file(file_path, new_contents)?;
    backup_engine.backup()?;

    restore_engine.restore_as_of_version(file_id, old_version)?;

    assert_target_file_contents(restore_target.path(), file_path, old_contents)?;
    Ok(())
}

// TODO: restore latest version by default
// TODO: deduplicate data

fn assert_target_file_contents(target: &Path, filename: &str, expected_contents: &str) -> Result<(), BakareError> {
    let restored_path = target.join(filename);
    let mut actual_contents = String::new();
    File::open(restored_path)?.read_to_string(&mut actual_contents)?;
    assert_eq!(expected_contents, actual_contents);
    Ok(())
}

fn assert_same_after_restore(source_path: &Path, repository: &Repository) -> Result<(), BakareError> {
    let backup_engine = backup::Engine::new(source_path, repository);
    backup_engine.backup()?;

    let restore_target = tempdir()?;
    let restore_engine = restore::Engine::new(repository, &restore_target.path());
    restore_engine.restore_all()?;

    let are_source_and_target_different = is_different(source_path, &restore_target.path()).unwrap();
    assert!(!are_source_and_target_different);
    Ok(())
}

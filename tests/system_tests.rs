use bakare::backup;
use bakare::restore;

use bakare::source::TempSource;

use bakare::error::BakareError;
use bakare::repository::Repository;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tempfile::tempdir;
use walkdir::WalkDir;

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
    let restore_engine = restore::Engine::new(&restore_repository, &restore_target.path())?;
    let old_contents = "some old contents";

    {
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source.path(), &mut backup_repository);
        source.write_text_to_file(relative_path_text, old_contents)?;
        backup_engine.backup()?;
    }

    let reading_repository = Repository::open(repository_path.as_path())?;
    let file_id = reading_repository.item_by_source_path(&file_path)?;
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
    let restore_target = tempdir()?.into_path();

    assert_ne!(source_path, repository_path);
    assert_ne!(repository_path, restore_target);

    {
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source_path, &mut backup_repository);
        backup_engine.backup()?;
    }
    {
        let restore_repository = Repository::open(repository_path.as_path())?;
        let restore_engine = restore::Engine::new(&restore_repository, &restore_target)?;
        restore_engine.restore_all()?;
    }

    assert_directory_trees_have_same_contents(source_path, restore_target.as_path())?;
    Ok(())
}

fn assert_directory_trees_have_same_contents(left: &Path, right: &Path) -> Result<(), BakareError> {
    let left_files = get_sorted_files_recursively(left)?;
    let right_files = get_sorted_files_recursively(right)?;

    let pairs = left_files.iter().zip(right_files);
    for (l, r) in pairs {
        assert_eq!(l.file_name(), r.file_name());
        let mut fl = File::open(l)?;
        let mut fr = File::open(r)?;
        let mut bl = vec![];
        let mut br = vec![];
        fl.read_to_end(&mut bl)?;
        fr.read_to_end(&mut br)?;
        assert_eq!(bl, bl);
    }
    Ok(())
}

fn get_sorted_files_recursively(path: &Path) -> Result<Vec<Box<Path>>, BakareError> {
    let walker = WalkDir::new(path).sort_by(|a, b| a.file_name().cmp(b.file_name()));

    let mut result = vec![];

    for maybe_entry in walker {
        let entry = maybe_entry?;
        if entry.path() == path {
            continue;
        }
        if entry.path().is_file() {
            result.push(Box::from(entry.path()));
        }
    }

    Ok(result)
}
// TODO: restore latest version by default
// TODO: deduplicate data

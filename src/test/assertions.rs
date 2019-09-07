use crate::error::BakareError;
use crate::repository::Repository;
use crate::{backup, restore};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tempfile::tempdir;
use walkdir::WalkDir;

pub fn assert_target_file_contents(restored_path: &Path, expected_contents: &str) -> Result<(), BakareError> {
    let mut actual_contents = String::new();
    assert!(restored_path.exists(), "Expected '{}' to be there", restored_path.display());
    File::open(restored_path)?.read_to_string(&mut actual_contents)?;
    assert_eq!(expected_contents, actual_contents);
    Ok(())
}

pub fn assert_same_after_restore(source_path: &Path) -> Result<(), BakareError> {
    let repository_path = tempdir()?.into_path();
    let restore_target = tempdir()?.into_path();

    assert_ne!(source_path, repository_path);
    assert_ne!(repository_path, restore_target);

    Repository::init(repository_path.as_path())?;
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
        assert_eq!(bl, br);
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

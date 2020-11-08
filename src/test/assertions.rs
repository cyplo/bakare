use std::fs::File;
use std::io::Read;
use std::path::Path;

use tempfile::tempdir;
use walkdir::WalkDir;

use super::source::TempSource;
use crate::repository::{item::RepositoryItem, ItemId, Repository};
use crate::{backup, restore};
use anyhow::Result;

pub fn assert_same_after_restore(source_path: &Path) -> Result<()> {
    let repository_path = tempdir().unwrap().into_path();
    let restore_target = tempdir().unwrap().into_path();

    assert_ne!(source_path, repository_path);
    assert_ne!(repository_path, restore_target);

    Repository::init(repository_path.as_path())?;
    {
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source_path, &mut backup_repository)?;
        backup_engine.backup()?;
    }
    {
        let mut restore_repository = Repository::open(repository_path.as_path())?;
        let mut restore_engine = restore::Engine::new(&mut restore_repository, &restore_target)?;
        restore_engine.restore_all()?;
    }

    assert_directory_trees_have_same_contents(source_path, restore_target.as_path())?;
    Ok(())
}

pub fn assert_restored_file_contents(repository_path: &Path, source_file_full_path: &Path, contents: &str) -> Result<()> {
    let mut restore_repository = Repository::open(repository_path)?;
    let item = restore_repository.newest_item_by_source_path(&source_file_full_path)?;
    let restore_target = tempdir().unwrap();
    let restore_engine = restore::Engine::new(&mut restore_repository, &restore_target.path())?;

    restore_engine.restore(&item.unwrap())?;
    let restored_file_path = restore_target.path().join(source_file_full_path.strip_prefix("/")?);
    assert_target_file_contents(&restored_file_path, contents)
}

pub fn assert_restored_from_version_has_contents(
    repository_path: &Path,
    source_file_full_path: &Path,
    old_contents: &str,
    old_id: &ItemId,
) -> Result<()> {
    let mut restore_repository = Repository::open(repository_path)?;
    let old_item = restore_repository.item_by_id(&old_id)?;
    let restore_target = tempdir().unwrap();
    let restore_engine = restore::Engine::new(&mut restore_repository, &restore_target.path())?;
    restore_engine.restore(&old_item.unwrap())?;
    let restored_file_path = restore_target.path().join(source_file_full_path.strip_prefix("/")?);
    assert_target_file_contents(&restored_file_path, old_contents)
}

pub fn newest_item(repository_path: &Path, source_file_full_path: &Path) -> Result<RepositoryItem> {
    let item = {
        let reading_repository = Repository::open(repository_path)?;
        let item = reading_repository.newest_item_by_source_path(&source_file_full_path)?;
        assert!(item.is_some());
        item.unwrap()
    };
    Ok(item)
}

pub fn restore_all_from_reloaded_repository(repository_path: &Path, restore_target: &Path) -> Result<()> {
    {
        let mut restore_repository = Repository::open(repository_path)?;
        let mut restore_engine = restore::Engine::new(&mut restore_repository, &restore_target)?;
        restore_engine.restore_all()?;
        Ok(())
    }
}

pub fn backup_file_with_contents(
    source: &TempSource,
    repository_path: &Path,
    source_file_relative_path: &str,
    contents: &str,
) -> Result<()> {
    {
        let mut backup_repository = Repository::open(repository_path)?;
        let mut backup_engine = backup::Engine::new(source.path(), &mut backup_repository)?;
        source.write_text_to_file(source_file_relative_path, contents).unwrap();
        backup_engine.backup()?;
        Ok(())
    }
}

pub fn data_weight(repository_path: &Path) -> Result<u64> {
    {
        let repository = Repository::open(repository_path)?;
        Ok(repository.data_weight()?)
    }
}

fn assert_directory_trees_have_same_contents(left: &Path, right: &Path) -> Result<()> {
    let left_files = get_sorted_files_recursively(left)?;
    let right_files = get_sorted_files_recursively(right)?;

    let pairs = left_files.iter().zip(right_files);
    for (l, r) in pairs {
        assert_eq!(l.file_name(), r.file_name());
        let mut fl = File::open(l).unwrap();
        let mut fr = File::open(r).unwrap();
        let mut bl = vec![];
        let mut br = vec![];
        fl.read_to_end(&mut bl).unwrap();
        fr.read_to_end(&mut br).unwrap();
        assert_eq!(bl, br);
    }
    Ok(())
}

pub fn get_sorted_files_recursively<T: AsRef<Path>>(path: T) -> Result<Vec<Box<Path>>> {
    let walker = WalkDir::new(path.as_ref()).sort_by(|a, b| a.file_name().cmp(b.file_name()));

    let mut result = vec![];

    for maybe_entry in walker {
        let entry = maybe_entry?;
        if entry.path() == path.as_ref() {
            continue;
        }
        if entry.path().is_file() {
            result.push(Box::from(entry.path()));
        }
    }

    Ok(result)
}

fn assert_target_file_contents(restored_path: &Path, expected_contents: &str) -> Result<()> {
    let mut actual_contents = String::new();
    assert!(restored_path.exists(), "Expected '{}' to be there", restored_path.display());
    File::open(restored_path)
        .unwrap()
        .read_to_string(&mut actual_contents)
        .unwrap();
    assert_eq!(expected_contents, actual_contents);
    Ok(())
}

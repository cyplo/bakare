use std::fs;
use std::io;
use std::path::Path;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub struct Engine<'a> {
    source_path: &'a Path,
    repository_path: &'a Path,
}

impl<'a> Engine<'a> {
    pub fn new(source_path: &'a Path, repository_path: &'a Path) -> Self {
        Engine {
            source_path,
            repository_path,
        }
    }

    pub fn backup(&self) -> Result<(), io::Error> {
        let walker = WalkDir::new(self.source_path);
        for maybe_entry in walker {
            let entry = maybe_entry?;
            if entry.path() != self.source_path {
                self.process_entry(&entry)?;
            }
        }
        Ok(())
    }

    pub fn file_version(&self, path: &Path) -> u64 {
        0
    }

    fn process_entry(&self, entry: &DirEntry) -> Result<(), io::Error> {
        if entry.file_type().is_dir() {
            fs::create_dir(self.repository_path.join(entry.file_name()))?;
        }
        if entry.file_type().is_file() {
            fs::copy(entry.path(), self.repository_path.join(entry.file_name()))?;
        }
        Ok(())
    }
}
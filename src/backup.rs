use std::fs;
use std::io;
use std::path::Path;
use storage::Version;
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

    pub fn file_version(&self, path: &Path) -> Version {
        Version(0)
    }

    fn process_entry(&self, entry: &DirEntry) -> Result<(), io::Error> {
        // TODO: remember entry in index

        // TODO: store file data

        if entry.file_type().is_dir() {
            fs::create_dir(self.repository_path.join(entry.file_name()))?;
        }
        if entry.file_type().is_file() {
            fs::copy(entry.path(), self.repository_path.join(entry.file_name()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod should {

    #[test]
    fn store_file_where_index_tells_it() {
        // fake index, all files stores at the same path
        // backup
        // see if repo contains one file at the faked path
        assert!(false);
    }

}

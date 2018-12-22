use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use tempfile::TempDir;

pub struct Source {
    directory: TempDir,
}

impl Source {
    pub fn new() -> Result<Self, Error> {
        Ok(Self { directory: tempdir()? })
    }

    pub fn write_text_to_file(&self, filename: &str, text: &str) -> Result<(), Error> {
        Ok(File::create(self.directory.path().join(filename))?.write_all(text.as_bytes())?)
    }

    pub fn path(&self) -> &Path {
        self.directory.path()
    }
}

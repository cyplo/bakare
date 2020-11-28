use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use tempfile::tempdir;
use tempfile::TempDir;

pub struct TempSource {
    directory: TempDir,
}

impl TempSource {
    pub fn new() -> Result<Self, Error> {
        Ok(Self { directory: tempdir()? })
    }

    pub fn write_bytes_to_file(&self, filename: &str, bytes: &[u8]) -> Result<(), Error> {
        let path = self.file_path(filename);
        Ok(File::create(path)?.write_all(bytes)?)
    }

    pub fn write_text_to_file(&self, filename: &str, text: &str) -> Result<(), Error> {
        self.write_bytes_to_file(filename, text.as_bytes())
    }

    pub fn write_random_bytes_to_file(&self, filename: &str, size: usize) -> Result<(), Error> {
        let random_bytes: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
        self.write_bytes_to_file(filename, &random_bytes)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    pub fn file_path(&self, filename: &str) -> PathBuf {
        self.directory.path().join(filename)
    }
}

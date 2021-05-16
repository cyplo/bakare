use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use tempfile::{tempdir, TempDir};

pub struct TestSource {
    directory: TempDir,
}

impl TestSource {
    pub fn new() -> Result<Self> {
        let dir = tempdir()?;
        Ok(Self { directory: dir })
    }

    pub fn write_bytes_to_file(&self, filename: &str, bytes: &[u8]) -> Result<()> {
        let path = self.file_path(filename)?;
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        Ok(())
    }

    pub fn write_text_to_file(&self, filename: &str, text: &str) -> Result<()> {
        self.write_bytes_to_file(filename, text.as_bytes())
    }

    pub fn write_random_bytes_to_file(&self, filename: &str, size: u64) -> Result<()> {
        let random_bytes: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
        self.write_bytes_to_file(filename, &random_bytes)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.directory.path()
    }

    pub fn file_path(&self, filename: &str) -> Result<PathBuf> {
        let file_path = self.directory.path().join(filename);
        Ok(file_path)
    }
}

impl Drop for TestSource {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.path());
    }
}
#[cfg(test)]
mod must {
    use super::TestSource;
    use anyhow::Result;

    #[test]
    fn leave_no_trace() -> Result<()> {
        let path;
        {
            let source = TestSource::new()?;
            source.write_random_bytes_to_file("somefile", 1)?;
            path = source.path().to_path_buf();
        }

        assert!(!path.exists());
        Ok(())
    }
}

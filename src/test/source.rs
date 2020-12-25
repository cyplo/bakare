use std::io::Write;

use anyhow::Result;
use vfs::VfsPath;

use super::assertions::in_memory::random_in_memory_path;

pub struct TestSource {
    directory: VfsPath,
}

impl TestSource {
    pub fn new() -> Result<Self> {
        let path: VfsPath = random_in_memory_path("testsource")?;
        path.create_dir_all()?;
        Ok(Self { directory: path })
    }

    pub fn write_bytes_to_file(&self, filename: &str, bytes: &[u8]) -> Result<()> {
        let path = self.file_path(filename)?;
        let mut file = path.create_file()?;
        file.write_all(bytes)?;
        dbg!(format!("wrote bytes under {}", filename));
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

    pub fn path(&self) -> &VfsPath {
        &self.directory
    }

    pub fn file_path(&self, filename: &str) -> Result<VfsPath> {
        let file_path = self.directory.join(filename)?;
        Ok(file_path)
    }
}

impl Drop for TestSource {
    fn drop(&mut self) {
        let _ = self.path().remove_dir_all();
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
            path = source.path().clone();
        }

        assert!(!path.exists());
        Ok(())
    }
}

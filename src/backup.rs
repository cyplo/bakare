use crate::repository::Repository;
use anyhow::Result;
use anyhow::*;
use vfs::VfsPath;

pub struct Engine<'a> {
    source_path: &'a VfsPath,
    repository: &'a mut Repository,
}

impl<'a> Engine<'a> {
    pub fn new(source_path: &'a VfsPath, repository: &'a mut Repository) -> Result<Self> {
        let mut ancestors = vec![];
        let mut current = Some(source_path.clone());
        while let Some(path) = current {
            ancestors.push(path.clone());
            current = path.parent();
        }
        if ancestors.into_iter().any(|a| &a == repository.path()) {
            return Err(anyhow!("source same as repository"));
        }
        Ok(Engine { source_path, repository })
    }

    pub fn backup(&mut self) -> Result<()> {
        let walker = self.source_path.walk_dir()?;
        let save_every = 16;
        let mut save_counter = 0;
        for maybe_entry in walker {
            let entry = maybe_entry?;
            if &entry != self.source_path {
                self.repository.store(&entry)?;
            }
            save_counter += 1;
            if save_counter == save_every {
                save_counter = 0;
                self.repository.save_index()?;
            }
        }
        self.repository.save_index()?;
        Ok(())
    }
}

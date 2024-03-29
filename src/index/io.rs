use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce}; // Or `XChaCha20Poly1305`

use uuid::Uuid;

use crate::index::item::IndexItem;
use crate::index::{lock, Index};
use crate::io::error_correcting_encoder;
use crate::repository::ItemId;
use anyhow::Result;
use anyhow::*;
use lock::Lock;
use nix::unistd::getpid;
use std::{cmp::max, io::Write};

impl Index {
    pub fn load(repository_path: &Path, secret: &[u8]) -> Result<Self> {
        if !repository_path.exists() {
            let mut index = Index::new()?;
            index.save(repository_path, secret)?;
        }
        let lock = Lock::lock(repository_path)?;
        let index_file_path = &Index::index_file_path_for_repository_path(repository_path)?;
        let index = Index::load_from_file(index_file_path, secret)?;
        lock.release()?;
        log::debug!(
            "[{}] loaded index from {}, version: {}; {} items",
            getpid(),
            index_file_path.to_string_lossy(),
            index.version,
            index.newest_items_by_source_path.len()
        );
        Ok(index)
    }

    pub fn save(&mut self, repository_path: &Path, secret: &[u8]) -> Result<()> {
        let lock_id = Uuid::new_v4();
        let lock = Lock::lock(repository_path)?;

        let index_file_path = &Index::index_file_path_for_repository_path(repository_path)?;
        if index_file_path.exists() {
            let index = Index::load_from_file(&Index::index_file_path_for_repository_path(repository_path)?, secret)?;
            self.merge_items_by_file_id(index.items_by_file_id);
            self.merge_newest_items(index.newest_items_by_source_path);
            self.version = max(self.version, index.version);
        }
        self.version = self.version.next();
        self.write_index_to_file(index_file_path, secret)?;
        lock.release()?;
        log::debug!(
            "[{}] saved index version {} with lock id {} to {}; {} items",
            getpid(),
            self.version,
            lock_id,
            index_file_path.to_string_lossy(),
            self.newest_items_by_source_path.len()
        );
        Ok(())
    }

    fn write_index_to_file(&mut self, index_file_path: &Path, secret: &[u8]) -> Result<()> {
        let parent = index_file_path.parent();
        match parent {
            None => Err(anyhow!(format!(
                "cannot get parent for {}",
                index_file_path.to_string_lossy()
            ))),
            Some(parent) => Ok(fs::create_dir_all(parent)),
        }??;

        let serialised = serde_json::to_string_pretty(&self)?;

        let bytes = serialised.as_bytes();

        let mut hash = [0; 32];
        blake::hash(256, secret, &mut hash)?;
        let key = Key::from_slice(&hash);
        let cipher = XChaCha20Poly1305::new(key);

        blake::hash(256, index_file_path.as_os_str().as_bytes(), &mut hash)?;
        let nonce = XNonce::from_slice(&hash[0..(192 / 8)]);

        let encrypted = cipher.encrypt(nonce, bytes.as_ref()).map_err(|e| anyhow!("{}", e))?;
        let encoded = error_correcting_encoder::encode(&encrypted)?;

        {
            let mut file = File::create(index_file_path)?;
            file.write_all(&encoded).context("writing index to disk")?;
            file.flush()?;
        }

        let readback = {
            let mut file = File::open(index_file_path)?;
            let mut readback = vec![];
            file.read_to_end(&mut readback)?;
            readback
        };

        if readback != encoded {
            Err(anyhow!("index readback incorrect"))
        } else {
            Ok(())
        }
    }

    fn load_from_file(index_file_path: &Path, secret: &[u8]) -> Result<Self> {
        let mut file = File::open(index_file_path)?;
        let mut encoded = vec![];
        file.read_to_end(&mut encoded)?;

        let decoded = error_correcting_encoder::decode(&encoded)?;

        let mut hash = [0; 32];
        blake::hash(256, secret, &mut hash)?;
        let key = Key::from_slice(&hash);
        let cipher = XChaCha20Poly1305::new(key);
        blake::hash(256, index_file_path.as_os_str().as_bytes(), &mut hash)?;
        let nonce = XNonce::from_slice(&hash[0..(192 / 8)]);

        let decrypted = cipher.decrypt(nonce, decoded.as_ref()).map_err(|e| anyhow!("{}", e))?;
        let index_text = String::from_utf8(decrypted)?;

        let index: Index = serde_json::from_str(&index_text)
            .context(format!("cannot read index from: {}", index_file_path.to_string_lossy()))?;
        Ok(index)
    }

    fn merge_newest_items(&mut self, old_newest_items: HashMap<String, IndexItem>) {
        for (source_path, old_newest_item) in old_newest_items {
            if let Some(new_newest_item) = self.newest_items_by_source_path.get(&source_path) {
                if old_newest_item.version() > new_newest_item.version() {
                    self.newest_items_by_source_path.insert(source_path, old_newest_item);
                }
            } else {
                self.newest_items_by_source_path.insert(source_path, old_newest_item);
            }
        }
    }

    fn merge_items_by_file_id(&mut self, old_items_by_file_id: HashMap<ItemId, IndexItem>) {
        self.items_by_file_id.extend(old_items_by_file_id);
    }

    fn index_file_path_for_repository_path(path: &Path) -> Result<PathBuf> {
        Ok(path.join("index"))
    }
}

#[cfg(test)]
mod must {
    use crate::index::Index;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn have_version_increased_when_saved() -> Result<()> {
        let temp_dir = tempdir()?;
        let mut index = Index::new()?;
        let old_version = index.version;

        let secret = b"some secret";
        index.save(temp_dir.path(), secret)?;

        let new_version = index.version;

        assert!(new_version > old_version);

        Ok(())
    }

    #[test]
    fn be_same_when_loaded_from_disk() -> Result<()> {
        let repository_path = tempdir()?;
        let mut original = Index::new()?;

        let secret = b"some secret";
        original.save(repository_path.path(), secret)?;
        let loaded = Index::load(repository_path.path(), secret)?;

        assert_eq!(original, loaded);

        Ok(())
    }
}

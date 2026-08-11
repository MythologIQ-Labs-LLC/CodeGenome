use std::path::{Path, PathBuf};

use crate::identity::address_of;

/// Per-file parse cache backed by bincode on disk.
pub struct FileCache {
    root: PathBuf,
}

/// A cached parse result with its content hash for validation.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CachedEntry {
    pub content_hash: String,
    pub nodes_bin: Vec<u8>,
    pub edges_bin: Vec<u8>,
}

impl FileCache {
    pub fn new(store_dir: &Path) -> Self {
        let root = store_dir.join("file_cache");
        let _ = std::fs::create_dir_all(&root);
        Self { root }
    }

    /// Load cached parse result. Returns None on miss or hash mismatch.
    pub fn get(&self, path: &Path, current_hash: &str) -> Option<CachedEntry> {
        let cache_path = self.cache_path(path);
        let data = std::fs::read(&cache_path).ok()?;
        let entry: CachedEntry = bincode::deserialize(&data).ok()?;
        if entry.content_hash == current_hash {
            Some(entry)
        } else {
            None
        }
    }

    /// Store a parse result keyed by source file path.
    pub fn put(&self, path: &Path, entry: &CachedEntry) -> Result<(), String> {
        let cache_path = self.cache_path(path);
        let data = bincode::serialize(entry).map_err(|e| e.to_string())?;
        std::fs::write(cache_path, data).map_err(|e| e.to_string())
    }

    fn cache_path(&self, path: &Path) -> PathBuf {
        let key = format!("{}", path.display());
        let hash = address_of(key.as_bytes());
        self.root.join(format!("{:?}.bin", hash))
    }
}

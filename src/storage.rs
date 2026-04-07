use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StorageBlobSyncClass {
    #[default]
    LocalOnly,
    CloudReplicated,
    Cache,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBlobMetadata {
    pub path: String,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub sync_class: StorageBlobSyncClass,
    pub byte_size: u64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageBlob {
    pub metadata: StorageBlobMetadata,
    pub bytes: Vec<u8>,
}

pub trait PluginStorage: Send + Sync {
    fn data_dir(&self, extension_id: &str) -> PathBuf;
    fn get(&self, extension_id: &str, key: &str) -> Result<Option<Value>, StorageError>;
    fn set(&mut self, extension_id: &str, key: &str, value: Value) -> Result<(), StorageError>;
    fn remove(&mut self, extension_id: &str, key: &str) -> Result<Option<Value>, StorageError>;
    fn keys(&self, extension_id: &str, prefix: Option<&str>) -> Result<Vec<String>, StorageError>;
    fn get_blob(&self, extension_id: &str, path: &str)
        -> Result<Option<StorageBlob>, StorageError>;
    fn put_blob(
        &mut self,
        extension_id: &str,
        path: &str,
        bytes: &[u8],
        content_type: Option<String>,
        sync_class: StorageBlobSyncClass,
        updated_at: String,
    ) -> Result<StorageBlobMetadata, StorageError>;
    fn delete_blob(&mut self, extension_id: &str, path: &str) -> Result<bool, StorageError>;
    fn list_blobs(
        &self,
        extension_id: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<StorageBlobMetadata>, StorageError>;
    fn stat_blob(
        &self,
        extension_id: &str,
        path: &str,
    ) -> Result<Option<StorageBlobMetadata>, StorageError>;
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("storage JSON failed: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("blob path '{0}' must be a safe relative path inside the extension namespace")]
    InvalidBlobPath(String),
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryExtensionStore {
    namespaces: HashMap<String, HashMap<String, Value>>,
    blob_namespaces: HashMap<String, HashMap<String, StorageBlob>>,
}

impl InMemoryExtensionStore {
    pub fn get(&self, extension_id: &str, key: &str) -> Option<&Value> {
        self.namespaces.get(extension_id).and_then(|ns| ns.get(key))
    }

    pub fn set(&mut self, extension_id: &str, key: impl Into<String>, value: Value) {
        self.namespaces
            .entry(extension_id.to_string())
            .or_default()
            .insert(key.into(), value);
    }

    pub fn remove(&mut self, extension_id: &str, key: &str) -> Option<Value> {
        self.namespaces
            .get_mut(extension_id)
            .and_then(|ns| ns.remove(key))
    }

    pub fn namespace(&self, extension_id: &str) -> Option<&HashMap<String, Value>> {
        self.namespaces.get(extension_id)
    }
}

impl PluginStorage for InMemoryExtensionStore {
    fn data_dir(&self, extension_id: &str) -> PathBuf {
        PathBuf::from(extension_id)
    }

    fn get(&self, extension_id: &str, key: &str) -> Result<Option<Value>, StorageError> {
        Ok(self.get(extension_id, key).cloned())
    }

    fn set(&mut self, extension_id: &str, key: &str, value: Value) -> Result<(), StorageError> {
        self.set(extension_id, key.to_string(), value);
        Ok(())
    }

    fn remove(&mut self, extension_id: &str, key: &str) -> Result<Option<Value>, StorageError> {
        Ok(self.remove(extension_id, key))
    }

    fn keys(&self, extension_id: &str, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        let mut keys = self
            .namespace(extension_id)
            .map(|namespace| namespace.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        if let Some(prefix) = prefix {
            keys.retain(|key| key.starts_with(prefix));
        }
        keys.sort();
        Ok(keys)
    }

    fn get_blob(
        &self,
        extension_id: &str,
        path: &str,
    ) -> Result<Option<StorageBlob>, StorageError> {
        let normalized = normalize_blob_path(path)?;
        Ok(self
            .blob_namespaces
            .get(extension_id)
            .and_then(|namespace| namespace.get(&normalized).cloned()))
    }

    fn put_blob(
        &mut self,
        extension_id: &str,
        path: &str,
        bytes: &[u8],
        content_type: Option<String>,
        sync_class: StorageBlobSyncClass,
        updated_at: String,
    ) -> Result<StorageBlobMetadata, StorageError> {
        let normalized = normalize_blob_path(path)?;
        let metadata = StorageBlobMetadata {
            path: normalized.clone(),
            content_type,
            sync_class,
            byte_size: bytes.len() as u64,
            updated_at,
        };
        self.blob_namespaces
            .entry(extension_id.to_string())
            .or_default()
            .insert(
                normalized,
                StorageBlob {
                    metadata: metadata.clone(),
                    bytes: bytes.to_vec(),
                },
            );
        Ok(metadata)
    }

    fn delete_blob(&mut self, extension_id: &str, path: &str) -> Result<bool, StorageError> {
        let normalized = normalize_blob_path(path)?;
        Ok(self
            .blob_namespaces
            .get_mut(extension_id)
            .and_then(|namespace| namespace.remove(&normalized))
            .is_some())
    }

    fn list_blobs(
        &self,
        extension_id: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<StorageBlobMetadata>, StorageError> {
        let normalized_prefix = match prefix {
            Some(value) => Some(normalize_blob_prefix(value)?),
            None => None,
        };
        let mut blobs = self
            .blob_namespaces
            .get(extension_id)
            .map(|namespace| {
                namespace
                    .values()
                    .map(|blob| blob.metadata.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if let Some(prefix) = normalized_prefix.as_deref() {
            blobs.retain(|blob| blob.path.starts_with(prefix));
        }
        blobs.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(blobs)
    }

    fn stat_blob(
        &self,
        extension_id: &str,
        path: &str,
    ) -> Result<Option<StorageBlobMetadata>, StorageError> {
        Ok(self.get_blob(extension_id, path)?.map(|blob| blob.metadata))
    }
}

#[derive(Debug, Clone)]
pub struct DiskExtensionStore {
    root: PathBuf,
}

impl DiskExtensionStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn namespace_dir(&self, extension_id: &str) -> PathBuf {
        self.root.join(extension_id)
    }

    fn namespace_file(&self, extension_id: &str) -> PathBuf {
        self.namespace_dir(extension_id).join("storage.json")
    }

    fn blobs_dir(&self, extension_id: &str) -> PathBuf {
        self.namespace_dir(extension_id).join("blobs")
    }

    fn blob_index_file(&self, extension_id: &str) -> PathBuf {
        self.namespace_dir(extension_id).join("blobs.json")
    }

    fn read_namespace(&self, extension_id: &str) -> Result<HashMap<String, Value>, StorageError> {
        let file = self.namespace_file(extension_id);
        if !file.exists() {
            return Ok(HashMap::new());
        }

        let bytes = fs::read(file)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn write_namespace(
        &self,
        extension_id: &str,
        namespace: &HashMap<String, Value>,
    ) -> Result<(), StorageError> {
        let dir = self.namespace_dir(extension_id);
        fs::create_dir_all(&dir)?;
        let file = self.namespace_file(extension_id);
        fs::write(file, serde_json::to_vec_pretty(namespace)?)?;
        Ok(())
    }

    fn read_blob_index(
        &self,
        extension_id: &str,
    ) -> Result<HashMap<String, StorageBlobMetadata>, StorageError> {
        let file = self.blob_index_file(extension_id);
        if !file.exists() {
            return Ok(HashMap::new());
        }

        let bytes = fs::read(file)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn write_blob_index(
        &self,
        extension_id: &str,
        index: &HashMap<String, StorageBlobMetadata>,
    ) -> Result<(), StorageError> {
        let dir = self.namespace_dir(extension_id);
        fs::create_dir_all(&dir)?;
        let file = self.blob_index_file(extension_id);
        fs::write(file, serde_json::to_vec_pretty(index)?)?;
        Ok(())
    }

    fn blob_data_path(&self, extension_id: &str, path: &str) -> Result<PathBuf, StorageError> {
        let normalized = normalize_blob_path(path)?;
        let mut resolved = self.blobs_dir(extension_id);
        for segment in normalized.split('/') {
            resolved.push(segment);
        }
        Ok(resolved)
    }
}

impl PluginStorage for DiskExtensionStore {
    fn data_dir(&self, extension_id: &str) -> PathBuf {
        self.namespace_dir(extension_id)
    }

    fn get(&self, extension_id: &str, key: &str) -> Result<Option<Value>, StorageError> {
        Ok(self.read_namespace(extension_id)?.remove(key))
    }

    fn set(&mut self, extension_id: &str, key: &str, value: Value) -> Result<(), StorageError> {
        let mut namespace = self.read_namespace(extension_id)?;
        namespace.insert(key.to_string(), value);
        self.write_namespace(extension_id, &namespace)
    }

    fn remove(&mut self, extension_id: &str, key: &str) -> Result<Option<Value>, StorageError> {
        let mut namespace = self.read_namespace(extension_id)?;
        let removed = namespace.remove(key);
        self.write_namespace(extension_id, &namespace)?;
        Ok(removed)
    }

    fn keys(&self, extension_id: &str, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        let mut keys = self
            .read_namespace(extension_id)?
            .into_keys()
            .collect::<Vec<_>>();
        if let Some(prefix) = prefix {
            keys.retain(|key| key.starts_with(prefix));
        }
        keys.sort();
        Ok(keys)
    }

    fn get_blob(
        &self,
        extension_id: &str,
        path: &str,
    ) -> Result<Option<StorageBlob>, StorageError> {
        let normalized = normalize_blob_path(path)?;
        let index = self.read_blob_index(extension_id)?;
        let Some(metadata) = index.get(&normalized).cloned() else {
            return Ok(None);
        };
        let bytes = fs::read(self.blob_data_path(extension_id, &normalized)?)?;
        Ok(Some(StorageBlob { metadata, bytes }))
    }

    fn put_blob(
        &mut self,
        extension_id: &str,
        path: &str,
        bytes: &[u8],
        content_type: Option<String>,
        sync_class: StorageBlobSyncClass,
        updated_at: String,
    ) -> Result<StorageBlobMetadata, StorageError> {
        let normalized = normalize_blob_path(path)?;
        let mut index = self.read_blob_index(extension_id)?;
        let file = self.blob_data_path(extension_id, &normalized)?;
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file, bytes)?;

        let metadata = StorageBlobMetadata {
            path: normalized.clone(),
            content_type,
            sync_class,
            byte_size: bytes.len() as u64,
            updated_at,
        };
        index.insert(normalized, metadata.clone());
        self.write_blob_index(extension_id, &index)?;
        Ok(metadata)
    }

    fn delete_blob(&mut self, extension_id: &str, path: &str) -> Result<bool, StorageError> {
        let normalized = normalize_blob_path(path)?;
        let mut index = self.read_blob_index(extension_id)?;
        let removed = index.remove(&normalized).is_some();
        if removed {
            let file = self.blob_data_path(extension_id, &normalized)?;
            match fs::remove_file(file) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(StorageError::Io(error)),
            }
            self.write_blob_index(extension_id, &index)?;
        }
        Ok(removed)
    }

    fn list_blobs(
        &self,
        extension_id: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<StorageBlobMetadata>, StorageError> {
        let normalized_prefix = match prefix {
            Some(value) => Some(normalize_blob_prefix(value)?),
            None => None,
        };
        let mut blobs = self
            .read_blob_index(extension_id)?
            .into_values()
            .collect::<Vec<_>>();
        if let Some(prefix) = normalized_prefix.as_deref() {
            blobs.retain(|blob| blob.path.starts_with(prefix));
        }
        blobs.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(blobs)
    }

    fn stat_blob(
        &self,
        extension_id: &str,
        path: &str,
    ) -> Result<Option<StorageBlobMetadata>, StorageError> {
        let normalized = normalize_blob_path(path)?;
        Ok(self.read_blob_index(extension_id)?.remove(&normalized))
    }
}

fn normalize_blob_path(path: &str) -> Result<String, StorageError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(StorageError::InvalidBlobPath(path.to_string()));
    }
    let candidate = Path::new(trimmed);
    let mut segments = Vec::new();
    for component in candidate.components() {
        match component {
            Component::Normal(segment) => {
                let value = segment.to_string_lossy();
                if value.is_empty() {
                    return Err(StorageError::InvalidBlobPath(path.to_string()));
                }
                segments.push(value.to_string());
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(StorageError::InvalidBlobPath(path.to_string()));
            }
        }
    }
    if segments.is_empty() {
        return Err(StorageError::InvalidBlobPath(path.to_string()));
    }
    Ok(segments.join("/"))
}

fn normalize_blob_prefix(prefix: &str) -> Result<String, StorageError> {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    normalize_blob_path(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn isolates_storage_by_extension() {
        let mut store = InMemoryExtensionStore::default();
        store.set("clipboard-tools", "provider", json!("duckduckgo"));
        store.set("browser-tools", "provider", json!("brave"));

        assert_eq!(
            store.get("clipboard-tools", "provider"),
            Some(&json!("duckduckgo"))
        );
        assert_eq!(
            store.get("browser-tools", "provider"),
            Some(&json!("brave"))
        );
    }

    #[test]
    fn persists_disk_storage_per_extension() {
        let root = tempdir().expect("temp dir");
        let mut store = DiskExtensionStore::new(root.path());

        store
            .set("music-tools", "provider", json!("musicbrainz"))
            .expect("write");
        store
            .set("music-tools", "last-query", json!("phoebe bridgers"))
            .expect("write");
        store
            .set("clipboard-tools", "provider", json!("brave"))
            .expect("write");

        assert_eq!(
            store.get("music-tools", "provider").expect("read"),
            Some(json!("musicbrainz"))
        );
        assert_eq!(
            store.keys("music-tools", None).expect("keys"),
            vec!["last-query".to_string(), "provider".to_string()]
        );
        assert_eq!(
            store.remove("music-tools", "provider").expect("remove"),
            Some(json!("musicbrainz"))
        );
        assert_eq!(
            store.get("clipboard-tools", "provider").expect("read"),
            Some(json!("brave"))
        );
    }

    #[test]
    fn persists_blob_storage_per_extension() {
        let root = tempdir().expect("temp dir");
        let mut store = DiskExtensionStore::new(root.path());

        let metadata = store
            .put_blob(
                "media-tools",
                "images/logo.png",
                b"hello world",
                Some("image/png".to_string()),
                StorageBlobSyncClass::CloudReplicated,
                "2026-03-27T00:00:00Z".to_string(),
            )
            .expect("write blob");

        assert_eq!(metadata.path, "images/logo.png");
        assert_eq!(metadata.byte_size, 11);

        let blob = store
            .get_blob("media-tools", "images/logo.png")
            .expect("get blob")
            .expect("blob exists");
        assert_eq!(blob.bytes, b"hello world");
        assert_eq!(
            blob.metadata.sync_class,
            StorageBlobSyncClass::CloudReplicated
        );

        assert_eq!(
            store
                .list_blobs("media-tools", Some("images"))
                .expect("list"),
            vec![metadata.clone()]
        );
        assert_eq!(
            store
                .stat_blob("media-tools", "images/logo.png")
                .expect("stat"),
            Some(metadata.clone())
        );
        assert!(store
            .delete_blob("media-tools", "images/logo.png")
            .expect("delete"));
        assert!(store
            .get_blob("media-tools", "images/logo.png")
            .expect("get missing")
            .is_none());
    }

    #[test]
    fn rejects_unsafe_blob_paths() {
        let mut store = InMemoryExtensionStore::default();
        let error = store
            .put_blob(
                "media-tools",
                "../secrets.txt",
                b"nope",
                None,
                StorageBlobSyncClass::LocalOnly,
                "2026-03-27T00:00:00Z".to_string(),
            )
            .expect_err("invalid path");
        assert!(matches!(error, StorageError::InvalidBlobPath(_)));
    }
}

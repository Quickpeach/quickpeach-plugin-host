use crate::manifest::{ExtensionManifestV1, ManifestValidationError, MANIFEST_FILE_NAME};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExtensionSource {
    LocalDirectory,
    NativeRegistry,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedExtension {
    pub manifest: ExtensionManifestV1,
    pub path: PathBuf,
    pub source: ExtensionSource,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryIssue {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveryReport {
    pub loaded: Vec<LoadedExtension>,
    pub issues: Vec<DiscoveryIssue>,
}

pub fn default_extensions_dir(home_dir: impl AsRef<Path>) -> PathBuf {
    home_dir.as_ref().join(".quickpeach").join("extensions")
}

pub fn discover_extensions(root: impl AsRef<Path>) -> DiscoveryReport {
    let root = root.as_ref();
    let mut loaded = Vec::new();
    let mut issues = Vec::new();

    if !root.exists() {
        return DiscoveryReport { loaded, issues };
    }

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            issues.push(DiscoveryIssue {
                path: root.to_path_buf(),
                message: error.to_string(),
            });
            return DiscoveryReport { loaded, issues };
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join(MANIFEST_FILE_NAME);
        if !manifest_path.exists() {
            continue;
        }

        match load_extension_dir(&path, ExtensionSource::LocalDirectory) {
            Ok(extension) => loaded.push(extension),
            Err(message) => issues.push(DiscoveryIssue { path, message }),
        }
    }

    DiscoveryReport { loaded, issues }
}

pub fn load_extension_dir(
    path: impl AsRef<Path>,
    source: ExtensionSource,
) -> Result<LoadedExtension, String> {
    let path = path.as_ref().to_path_buf();
    let manifest_path = path.join(MANIFEST_FILE_NAME);
    let raw = fs::read_to_string(&manifest_path).map_err(|error| error.to_string())?;
    let manifest: ExtensionManifestV1 =
        serde_json::from_str(&raw).map_err(|error| error.to_string())?;

    let folder_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if folder_name != manifest.id {
        return Err(format!(
            "folder name '{}' does not match manifest id '{}'",
            folder_name, manifest.id
        ));
    }

    manifest.validate().map_err(format_validation_error)?;

    Ok(LoadedExtension {
        manifest,
        path,
        source,
        enabled: true,
    })
}

fn format_validation_error(error: ManifestValidationError) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{
        ExtensionCapability, ExtensionCommandActionV1, ExtensionCommandV1, ExtensionManifestV1,
        ExtensionPermission,
    };
    use std::collections::BTreeSet;
    use tempfile::tempdir;

    #[test]
    fn discovers_valid_extensions_and_reports_invalid_ones() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let valid_dir = root.join("clipboard-tools");
        fs::create_dir_all(&valid_dir).expect("create valid dir");
        let valid_manifest = ExtensionManifestV1 {
            schema_version: 1,
            id: "clipboard-tools".to_string(),
            name: "Clipboard Tools".to_string(),
            description: "Clipboard helpers".to_string(),
            version: "0.1.0".to_string(),
            author: None,
            icon: None,
            commands: vec![ExtensionCommandV1 {
                id: "copy".to_string(),
                name: "Copy".to_string(),
                description: "Copies text".to_string(),
                keywords: vec![],
                action: ExtensionCommandActionV1::ClipboardWrite {
                    text: "hello".to_string(),
                },
            }],
            view: None,
            permissions: BTreeSet::from([ExtensionPermission::ClipboardWrite]),
            capabilities: BTreeSet::from([
                ExtensionCapability::LauncherCommands,
                ExtensionCapability::Clipboard,
            ]),
            preferences: vec![],
            platform: None,
        };
        fs::write(
            valid_dir.join(MANIFEST_FILE_NAME),
            serde_json::to_vec_pretty(&valid_manifest).expect("serialize"),
        )
        .expect("write valid manifest");

        let invalid_dir = root.join("bad-manifest");
        fs::create_dir_all(&invalid_dir).expect("create invalid dir");
        fs::write(invalid_dir.join(MANIFEST_FILE_NAME), b"{\"id\":")
            .expect("write invalid manifest");

        let report = discover_extensions(root);
        assert_eq!(report.loaded.len(), 1);
        assert_eq!(report.loaded[0].manifest.id, "clipboard-tools");
        assert_eq!(report.issues.len(), 1);
    }
}

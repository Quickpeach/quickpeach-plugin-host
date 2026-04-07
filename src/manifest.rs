use crate::bridge::{LauncherListItem, SearchProvider};
use crate::platform::PluginPlatformManifest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Component, Path};
use thiserror::Error;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const MANIFEST_FILE_NAME: &str = "manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExtensionCapability {
    LauncherCommands,
    Clipboard,
    Browser,
    Network,
    Ai,
    Calendar,
    Notes,
    Storage,
    Secrets,
    SyncState,
    Preferences,
    SearchFiles,
    Events,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExtensionPermission {
    ClipboardRead,
    ClipboardWrite,
    ClipboardHistory,
    BrowserOpen,
    BrowserSearch,
    NetworkFetch,
    AiInvoke,
    CalendarRead,
    CalendarWrite,
    CalendarSync,
    NotesRead,
    NotesWrite,
    StorageRead,
    StorageWrite,
    SecretsRead,
    SecretsWrite,
    SyncStateRead,
    SyncStateWrite,
    PreferencesRead,
    SearchFiles,
    Toast,
    Navigate,
    Events,
}

impl ExtensionPermission {
    pub fn required_capability(self) -> ExtensionCapability {
        match self {
            Self::ClipboardRead | Self::ClipboardWrite | Self::ClipboardHistory => {
                ExtensionCapability::Clipboard
            }
            Self::BrowserOpen | Self::BrowserSearch => ExtensionCapability::Browser,
            Self::NetworkFetch => ExtensionCapability::Network,
            Self::AiInvoke => ExtensionCapability::Ai,
            Self::CalendarRead | Self::CalendarWrite | Self::CalendarSync => {
                ExtensionCapability::Calendar
            }
            Self::NotesRead | Self::NotesWrite => ExtensionCapability::Notes,
            Self::StorageRead | Self::StorageWrite => ExtensionCapability::Storage,
            Self::SecretsRead | Self::SecretsWrite => ExtensionCapability::Secrets,
            Self::SyncStateRead | Self::SyncStateWrite => ExtensionCapability::SyncState,
            Self::PreferencesRead => ExtensionCapability::Preferences,
            Self::SearchFiles => ExtensionCapability::SearchFiles,
            Self::Toast | Self::Navigate => ExtensionCapability::LauncherCommands,
            Self::Events => ExtensionCapability::Events,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionViewV1 {
    pub entry: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default = "default_width")]
    pub width: u16,
    #[serde(default = "default_height")]
    pub height: u16,
    #[serde(default = "default_true")]
    pub resizable: bool,
}

fn default_width() -> u16 {
    640
}

fn default_height() -> u16 {
    520
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExtensionPreferenceKind {
    String,
    Boolean,
    Number,
    Enum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionPreferenceV1 {
    pub key: String,
    pub title: String,
    pub kind: ExtensionPreferenceKind,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<Value>,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ExtensionCommandActionV1 {
    OpenUrl {
        url: String,
    },
    SearchWeb {
        query: String,
        #[serde(default)]
        provider: SearchProvider,
    },
    ClipboardWrite {
        text: String,
    },
    EmitEvent {
        event: String,
        #[serde(default)]
        payload: Value,
    },
    AiPrompt {
        prompt: String,
        #[serde(default)]
        model: Option<String>,
    },
    SearchFiles {
        query: String,
        #[serde(default)]
        roots: Vec<String>,
        #[serde(default)]
        extensions: Vec<String>,
        #[serde(default)]
        include_hidden: bool,
        #[serde(default)]
        max_depth: Option<usize>,
        #[serde(default)]
        limit: Option<usize>,
    },
    CalendarSync {
        #[serde(default)]
        source_ids: Vec<String>,
    },
    CalendarListEvents {
        #[serde(default)]
        source_ids: Vec<String>,
        #[serde(default)]
        from: Option<String>,
        #[serde(default)]
        to: Option<String>,
        #[serde(default)]
        query: Option<String>,
        #[serde(default)]
        limit: Option<usize>,
    },
    ReturnItems {
        items: Vec<LauncherListItem>,
    },
    AsyncTask {
        task_id: String,
        label: String,
    },
}

impl ExtensionCommandActionV1 {
    pub fn required_permissions(&self) -> BTreeSet<ExtensionPermission> {
        let mut permissions = BTreeSet::new();
        match self {
            Self::OpenUrl { .. } => {
                permissions.insert(ExtensionPermission::BrowserOpen);
            }
            Self::SearchWeb { .. } => {
                permissions.insert(ExtensionPermission::BrowserSearch);
            }
            Self::ClipboardWrite { .. } => {
                permissions.insert(ExtensionPermission::ClipboardWrite);
            }
            Self::EmitEvent { .. } => {
                permissions.insert(ExtensionPermission::Events);
            }
            Self::AiPrompt { .. } => {
                permissions.insert(ExtensionPermission::AiInvoke);
            }
            Self::SearchFiles { .. } => {
                permissions.insert(ExtensionPermission::SearchFiles);
            }
            Self::CalendarSync { .. } => {
                permissions.insert(ExtensionPermission::CalendarSync);
            }
            Self::CalendarListEvents { .. } => {
                permissions.insert(ExtensionPermission::CalendarRead);
            }
            Self::ReturnItems { .. } | Self::AsyncTask { .. } => {}
        }
        permissions
    }

    pub fn required_capabilities(&self) -> BTreeSet<ExtensionCapability> {
        self.required_permissions()
            .into_iter()
            .map(ExtensionPermission::required_capability)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionCommandV1 {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub action: ExtensionCommandActionV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifestV1 {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub commands: Vec<ExtensionCommandV1>,
    #[serde(default)]
    pub view: Option<ExtensionViewV1>,
    #[serde(default)]
    pub permissions: BTreeSet<ExtensionPermission>,
    #[serde(default)]
    pub capabilities: BTreeSet<ExtensionCapability>,
    #[serde(default)]
    pub preferences: Vec<ExtensionPreferenceV1>,
    #[serde(default)]
    pub platform: Option<PluginPlatformManifest>,
}

fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestValidationError {
    #[error("unsupported schema version {0}")]
    UnsupportedSchema(u32),
    #[error("extension id must be lowercase kebab-case")]
    InvalidId,
    #[error("duplicate command id '{0}'")]
    DuplicateCommandId(String),
    #[error("duplicate preference key '{0}'")]
    DuplicatePreference(String),
    #[error("view entry '{0}' must be a safe relative path inside the extension directory")]
    InvalidViewEntry(String),
    #[error(
        "platform view entry '{0}' must be a safe relative path inside the extension directory"
    )]
    InvalidPlatformViewEntry(String),
    #[error(
        "platform asset path '{0}' must be a safe relative path inside the extension directory"
    )]
    InvalidPlatformAssetPath(String),
    #[error("command '{command}' requires permission '{permission:?}'")]
    MissingPermission {
        command: String,
        permission: ExtensionPermission,
    },
    #[error("permission '{permission:?}' requires capability '{capability:?}'")]
    MissingCapabilityForPermission {
        permission: ExtensionPermission,
        capability: ExtensionCapability,
    },
    #[error("command '{command}' requires capability '{capability:?}'")]
    MissingCapability {
        command: String,
        capability: ExtensionCapability,
    },
}

impl ExtensionManifestV1 {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedSchema(
                self.schema_version,
            ));
        }

        if !is_valid_extension_id(&self.id) {
            return Err(ManifestValidationError::InvalidId);
        }

        if let Some(view) = &self.view {
            if !is_safe_relative_path(&view.entry) {
                return Err(ManifestValidationError::InvalidViewEntry(
                    view.entry.clone(),
                ));
            }
        }

        if let Some(platform) = &self.platform {
            for view in &platform.views {
                if !is_safe_relative_path(&view.entry) {
                    return Err(ManifestValidationError::InvalidPlatformViewEntry(
                        view.entry.clone(),
                    ));
                }
                for style in &view.styles {
                    if !is_safe_relative_path(&style.path) {
                        return Err(ManifestValidationError::InvalidPlatformAssetPath(
                            style.path.clone(),
                        ));
                    }
                }
            }

            for widget in &platform.dashboard_widgets {
                for style in &widget.styles {
                    if !is_safe_relative_path(&style.path) {
                        return Err(ManifestValidationError::InvalidPlatformAssetPath(
                            style.path.clone(),
                        ));
                    }
                }
            }
        }

        let mut command_ids = BTreeSet::new();
        for command in &self.commands {
            if !command_ids.insert(command.id.clone()) {
                return Err(ManifestValidationError::DuplicateCommandId(
                    command.id.clone(),
                ));
            }

            for permission in command.action.required_permissions() {
                if !self.permissions.contains(&permission) {
                    return Err(ManifestValidationError::MissingPermission {
                        command: command.id.clone(),
                        permission,
                    });
                }
            }

            for capability in command.action.required_capabilities() {
                if !self.capabilities.contains(&capability) {
                    return Err(ManifestValidationError::MissingCapability {
                        command: command.id.clone(),
                        capability,
                    });
                }
            }
        }

        for permission in &self.permissions {
            let capability = permission.required_capability();
            if !self.capabilities.contains(&capability) {
                return Err(ManifestValidationError::MissingCapabilityForPermission {
                    permission: *permission,
                    capability,
                });
            }
        }

        let mut preference_keys = BTreeSet::new();
        for preference in &self.preferences {
            if !preference_keys.insert(preference.key.clone()) {
                return Err(ManifestValidationError::DuplicatePreference(
                    preference.key.clone(),
                ));
            }
        }

        if let Some(platform) = &self.platform {
            for section in &platform.settings_sections {
                for field in &section.fields {
                    if !preference_keys.insert(field.key().to_string()) {
                        return Err(ManifestValidationError::DuplicatePreference(
                            field.key().to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

fn is_valid_extension_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn is_safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.trim().is_empty()
        && path.is_relative()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> ExtensionManifestV1 {
        ExtensionManifestV1 {
            schema_version: 1,
            id: "clipboard-tools".to_string(),
            name: "Clipboard Tools".to_string(),
            description: "Clipboard helpers".to_string(),
            version: "0.1.0".to_string(),
            author: None,
            icon: None,
            commands: vec![ExtensionCommandV1 {
                id: "copy-hello".to_string(),
                name: "Copy Hello".to_string(),
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
        }
    }

    #[test]
    fn validates_a_well_formed_manifest() {
        manifest().validate().expect("manifest should validate");
    }

    #[test]
    fn rejects_missing_permission() {
        let mut manifest = manifest();
        manifest.permissions.clear();

        let error = manifest.validate().expect_err("manifest should fail");
        assert!(matches!(
            error,
            ManifestValidationError::MissingPermission { command, permission }
                if command == "copy-hello" && permission == ExtensionPermission::ClipboardWrite
        ));
    }

    #[test]
    fn rejects_duplicate_preference_keys() {
        let mut manifest = manifest();
        manifest.preferences = vec![
            ExtensionPreferenceV1 {
                key: "provider".to_string(),
                title: "Provider".to_string(),
                kind: ExtensionPreferenceKind::String,
                description: None,
                required: false,
                default_value: None,
                options: vec![],
            },
            ExtensionPreferenceV1 {
                key: "provider".to_string(),
                title: "Provider Again".to_string(),
                kind: ExtensionPreferenceKind::String,
                description: None,
                required: false,
                default_value: None,
                options: vec![],
            },
        ];

        let error = manifest.validate().expect_err("manifest should fail");
        assert_eq!(
            error,
            ManifestValidationError::DuplicatePreference("provider".to_string())
        );
    }

    #[test]
    fn rejects_duplicate_platform_settings_keys() {
        let mut manifest = manifest();
        manifest.platform = Some(PluginPlatformManifest {
            settings_sections: vec![crate::platform::PluginSettingsSection {
                plugin_id: "clipboard-tools".to_string(),
                section_id: "clipboard".to_string(),
                title: "Clipboard".to_string(),
                icon: None,
                fields: vec![crate::platform::PluginSettingField::String {
                    key: "provider".to_string(),
                    label: "Provider".to_string(),
                    description: None,
                    placeholder: None,
                    default: None,
                    secret: false,
                }],
            }],
            ..PluginPlatformManifest::default()
        });
        manifest.preferences = vec![ExtensionPreferenceV1 {
            key: "provider".to_string(),
            title: "Provider".to_string(),
            kind: ExtensionPreferenceKind::String,
            description: None,
            required: false,
            default_value: None,
            options: vec![],
        }];

        let error = manifest.validate().expect_err("manifest should fail");
        assert_eq!(
            error,
            ManifestValidationError::DuplicatePreference("provider".to_string())
        );
    }

    #[test]
    fn rejects_parent_directory_view_entries() {
        let mut manifest = manifest();
        manifest.view = Some(ExtensionViewV1 {
            entry: "../outside.html".to_string(),
            title: None,
            width: 640,
            height: 520,
            resizable: true,
        });

        let error = manifest.validate().expect_err("manifest should fail");
        assert_eq!(
            error,
            ManifestValidationError::InvalidViewEntry("../outside.html".to_string())
        );
    }

    #[test]
    fn rejects_parent_directory_platform_view_entries() {
        let mut manifest = manifest();
        manifest.platform = Some(PluginPlatformManifest {
            views: vec![crate::platform::PluginViewDefinition {
                plugin_id: "clipboard-tools".to_string(),
                view_id: "main".to_string(),
                entry: "../outside.js".to_string(),
                export_name: None,
                title: None,
                mount: crate::platform::ViewMount::Workspace,
                styles: vec![],
            }],
            ..PluginPlatformManifest::default()
        });

        let error = manifest.validate().expect_err("manifest should fail");
        assert_eq!(
            error,
            ManifestValidationError::InvalidPlatformViewEntry("../outside.js".to_string())
        );
    }
}

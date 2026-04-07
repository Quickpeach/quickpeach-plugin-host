use crate::{
    manifest::ExtensionPermission,
    storage::{StorageBlobMetadata, StorageBlobSyncClass},
};
pub use peachnote_vault_core::EncodedVaultEnvelope as CryptoEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SearchProvider {
    #[default]
    Default,
    DuckDuckGo,
    Google,
    Brave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltInView {
    Home,
    Notes,
    Settings,
    Dashboard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherListItem {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalendarSourceKind {
    IcsUrl,
    IcsFile,
    Native,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalendarConnectorKind {
    Native,
    Provider,
    Feed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalendarConnectorAuthMode {
    Os,
    Oauth,
    Url,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarConnectorDescriptor {
    pub id: String,
    pub title: String,
    pub description: String,
    pub kind: CalendarConnectorKind,
    pub auth_mode: CalendarConnectorAuthMode,
    pub platform: String,
    pub supported: bool,
    pub read_only: bool,
    pub requires_permission: bool,
    #[serde(default)]
    pub docs_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSourceInput {
    pub id: String,
    pub name: String,
    pub kind: CalendarSourceKind,
    pub location: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSource {
    pub id: String,
    pub name: String,
    pub kind: CalendarSourceKind,
    pub location: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub last_synced_at: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventTime {
    pub value: String,
    #[serde(default)]
    pub date_only: bool,
    #[serde(default)]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub source_id: String,
    pub title: String,
    pub start: CalendarEventTime,
    #[serde(default)]
    pub end: Option<CalendarEventTime>,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSyncResult {
    pub source_id: String,
    #[serde(default)]
    pub synced_at: Option<String>,
    #[serde(default)]
    pub events_count: usize,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkResponse {
    pub url: String,
    pub ok: bool,
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub body_text: String,
    #[serde(default)]
    pub json: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBlobRecord {
    pub metadata: StorageBlobMetadata,
    pub data_base64: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum BridgeRequest {
    ShowToast {
        message: String,
        level: ToastLevel,
    },
    Navigate {
        view: BuiltInView,
    },
    OpenWorkspaceScreen {
        screen: String,
    },
    OpenUtilityTool {
        tool: String,
    },
    NetworkFetch {
        url: String,
        #[serde(default)]
        method: Option<String>,
        #[serde(default)]
        headers: BTreeMap<String, String>,
        #[serde(default)]
        body: Option<Value>,
    },
    ProviderFetch {
        provider_id: String,
        path: String,
        #[serde(default)]
        method: Option<String>,
        #[serde(default)]
        headers: BTreeMap<String, String>,
        #[serde(default)]
        body: Option<Value>,
    },
    OpenOverlay {
        namespace: String,
        owner_id: String,
        view_id: String,
        #[serde(default)]
        params: Value,
    },
    HideOverlay {
        namespace: String,
        window_label: String,
    },
    OpenNote {
        note_id: String,
    },
    ListNotes,
    CreateNote {
        title: String,
        #[serde(default)]
        content: Option<String>,
    },
    ClipboardRead,
    ClipboardWrite {
        text: String,
    },
    ClipboardHistory {
        #[serde(default)]
        limit: Option<usize>,
    },
    BrowserOpen {
        url: String,
    },
    BrowserSearch {
        query: String,
        #[serde(default)]
        provider: SearchProvider,
    },
    AiInvoke {
        prompt: String,
        #[serde(default)]
        model: Option<String>,
    },
    StorageGet {
        key: String,
    },
    StorageBlobGet {
        path: String,
    },
    StorageKeys {
        #[serde(default)]
        prefix: Option<String>,
    },
    StorageBlobList {
        #[serde(default)]
        prefix: Option<String>,
    },
    StorageSet {
        key: String,
        value: Value,
    },
    StorageBlobPut {
        path: String,
        data_base64: String,
        #[serde(default)]
        content_type: Option<String>,
        #[serde(default)]
        sync_class: Option<StorageBlobSyncClass>,
    },
    StorageRemove {
        key: String,
    },
    StorageBlobDelete {
        path: String,
    },
    StorageBlobStat {
        path: String,
    },
    SecretGet {
        key: String,
    },
    SecretKeys {
        #[serde(default)]
        prefix: Option<String>,
    },
    SecretSet {
        key: String,
        value: Value,
    },
    SecretRemove {
        key: String,
    },
    SyncStateGet {
        key: String,
    },
    SyncStateKeys {
        #[serde(default)]
        prefix: Option<String>,
    },
    SyncStateSet {
        key: String,
        value: Value,
    },
    SyncStateRemove {
        key: String,
    },
    PreferenceGet {
        key: String,
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
    CalendarListConnectors,
    CalendarListSources,
    CalendarUpsertSource {
        source: CalendarSourceInput,
    },
    CalendarRemoveSource {
        source_id: String,
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
    EmitEvent {
        event: String,
        #[serde(default)]
        payload: Value,
    },
    CryptoSeal {
        scope: String,
        data_base64: String,
        #[serde(default)]
        aad_base64: Option<String>,
    },
    CryptoOpen {
        scope: String,
        envelope: CryptoEnvelope,
        #[serde(default)]
        aad_base64: Option<String>,
    },
}

impl BridgeRequest {
    pub fn required_permission(&self) -> Option<ExtensionPermission> {
        match self {
            Self::ShowToast { .. } => Some(ExtensionPermission::Toast),
            Self::Navigate { .. }
            | Self::OpenWorkspaceScreen { .. }
            | Self::OpenUtilityTool { .. }
            | Self::OpenOverlay { .. }
            | Self::HideOverlay { .. } => Some(ExtensionPermission::Navigate),
            Self::NetworkFetch { .. } | Self::ProviderFetch { .. } => {
                Some(ExtensionPermission::NetworkFetch)
            }
            Self::OpenNote { .. } | Self::ListNotes => Some(ExtensionPermission::NotesRead),
            Self::CreateNote { .. } => Some(ExtensionPermission::NotesWrite),
            Self::ClipboardRead => Some(ExtensionPermission::ClipboardRead),
            Self::ClipboardWrite { .. } => Some(ExtensionPermission::ClipboardWrite),
            Self::ClipboardHistory { .. } => Some(ExtensionPermission::ClipboardHistory),
            Self::BrowserOpen { .. } => Some(ExtensionPermission::BrowserOpen),
            Self::BrowserSearch { .. } => Some(ExtensionPermission::BrowserSearch),
            Self::AiInvoke { .. } => Some(ExtensionPermission::AiInvoke),
            Self::StorageGet { .. }
            | Self::StorageBlobGet { .. }
            | Self::StorageKeys { .. }
            | Self::StorageBlobList { .. }
            | Self::StorageBlobStat { .. }
            | Self::CryptoOpen { .. } => Some(ExtensionPermission::StorageRead),
            Self::StorageSet { .. }
            | Self::StorageBlobPut { .. }
            | Self::StorageRemove { .. }
            | Self::StorageBlobDelete { .. }
            | Self::CryptoSeal { .. } => Some(ExtensionPermission::StorageWrite),
            Self::SecretGet { .. } | Self::SecretKeys { .. } => {
                Some(ExtensionPermission::SecretsRead)
            }
            Self::SecretSet { .. } | Self::SecretRemove { .. } => {
                Some(ExtensionPermission::SecretsWrite)
            }
            Self::SyncStateGet { .. } | Self::SyncStateKeys { .. } => {
                Some(ExtensionPermission::SyncStateRead)
            }
            Self::SyncStateSet { .. } | Self::SyncStateRemove { .. } => {
                Some(ExtensionPermission::SyncStateWrite)
            }
            Self::PreferenceGet { .. } => Some(ExtensionPermission::PreferencesRead),
            Self::SearchFiles { .. } => Some(ExtensionPermission::SearchFiles),
            Self::CalendarListConnectors
            | Self::CalendarListSources
            | Self::CalendarListEvents { .. } => Some(ExtensionPermission::CalendarRead),
            Self::CalendarUpsertSource { .. } | Self::CalendarRemoveSource { .. } => {
                Some(ExtensionPermission::CalendarWrite)
            }
            Self::CalendarSync { .. } => Some(ExtensionPermission::CalendarSync),
            Self::EmitEvent { .. } => Some(ExtensionPermission::Events),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum BridgeResponse {
    Ack {
        #[serde(default)]
        message: Option<String>,
    },
    Notes {
        notes: Vec<NoteSummary>,
    },
    ClipboardText {
        text: String,
    },
    ClipboardHistory {
        entries: Vec<String>,
    },
    StorageValue {
        #[serde(default)]
        value: Option<Value>,
    },
    StorageBlobValue {
        #[serde(default)]
        value: Option<StorageBlobRecord>,
    },
    StorageKeys {
        keys: Vec<String>,
    },
    StorageBlobList {
        blobs: Vec<StorageBlobMetadata>,
    },
    StorageBlobStat {
        #[serde(default)]
        value: Option<StorageBlobMetadata>,
    },
    SecretValue {
        #[serde(default)]
        value: Option<Value>,
    },
    SecretKeys {
        keys: Vec<String>,
    },
    SyncStateValue {
        #[serde(default)]
        value: Option<Value>,
    },
    SyncStateKeys {
        keys: Vec<String>,
    },
    PreferenceValue {
        #[serde(default)]
        value: Option<Value>,
    },
    SearchResults {
        items: Vec<LauncherListItem>,
    },
    CalendarConnectors {
        connectors: Vec<CalendarConnectorDescriptor>,
    },
    CalendarSources {
        sources: Vec<CalendarSource>,
    },
    CalendarEvents {
        events: Vec<CalendarEvent>,
    },
    CalendarSync {
        results: Vec<CalendarSyncResult>,
    },
    AiResult {
        output: String,
    },
    CryptoSealed {
        envelope: CryptoEnvelope,
    },
    CryptoOpened {
        data_base64: String,
    },
    CreatedNote {
        note_id: String,
    },
    OpenedWindow {
        window_label: String,
    },
    NetworkResponse {
        response: NetworkResponse,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum HostEffect {
    Bridge { request: BridgeRequest },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_keys_requires_storage_read_permission() {
        assert_eq!(
            BridgeRequest::StorageKeys { prefix: None }.required_permission(),
            Some(ExtensionPermission::StorageRead)
        );
    }

    #[test]
    fn storage_blob_write_requires_storage_write_permission() {
        assert_eq!(
            BridgeRequest::StorageBlobPut {
                path: "files/demo.txt".to_string(),
                data_base64: "aGVsbG8=".to_string(),
                content_type: None,
                sync_class: None,
            }
            .required_permission(),
            Some(ExtensionPermission::StorageWrite)
        );
    }

    #[test]
    fn crypto_open_requires_storage_read_permission() {
        assert_eq!(
            BridgeRequest::CryptoOpen {
                scope: "storage".to_string(),
                envelope: CryptoEnvelope {
                    algorithm: "xchacha20poly1305".to_string(),
                    key_version: 1,
                    nonce_base64: "bm9uY2U=".to_string(),
                    ciphertext_base64: "Y2lwaGVy".to_string(),
                },
                aad_base64: None,
            }
            .required_permission(),
            Some(ExtensionPermission::StorageRead)
        );
    }

    #[test]
    fn secret_keys_requires_secrets_read_permission() {
        assert_eq!(
            BridgeRequest::SecretKeys { prefix: None }.required_permission(),
            Some(ExtensionPermission::SecretsRead)
        );
    }

    #[test]
    fn sync_state_set_requires_sync_state_write_permission() {
        assert_eq!(
            BridgeRequest::SyncStateSet {
                key: "worker".to_string(),
                value: Value::Null,
            }
            .required_permission(),
            Some(ExtensionPermission::SyncStateWrite)
        );
    }
}

pub mod bridge;
pub mod discovery;
pub mod importer;
pub mod manifest;
pub mod platform;
pub mod runtime;
pub mod storage;

pub use bridge::{
    BridgeRequest, BridgeResponse, BuiltInView, CalendarConnectorAuthMode,
    CalendarConnectorDescriptor, CalendarConnectorKind, CalendarEvent, CalendarEventTime,
    CalendarSource, CalendarSourceInput, CalendarSourceKind, CalendarSyncResult,
    CryptoEnvelope, HostEffect, LauncherListItem, NetworkResponse, NoteSummary,
    SearchProvider, StorageBlobRecord, ToastLevel,
};
pub use discovery::{
    default_extensions_dir, discover_extensions, load_extension_dir, DiscoveryIssue,
    DiscoveryReport, ExtensionSource, LoadedExtension,
};
pub use importer::{
    translate_external_repo, CompatibilityLevel, CompatibilityMatrix, ExternalExtensionDescriptor,
    ImportPreview, UpstreamSourceKind,
};
pub use manifest::{
    ExtensionCapability, ExtensionCommandActionV1, ExtensionCommandV1, ExtensionManifestV1,
    ExtensionPermission, ExtensionPreferenceKind, ExtensionPreferenceV1, ExtensionViewV1,
    ManifestValidationError, CURRENT_SCHEMA_VERSION, MANIFEST_FILE_NAME,
};
pub use platform::{
    DashboardPosition, DashboardRegistry, DashboardWidgetDefinition, OverlayFamilyRegistration,
    OverlayRegistryPlan, PluginDescriptor, PluginHost, PluginLauncherContract,
    PluginPlatformManifest, PluginProviderDescriptor, PluginSettingField,
    PluginSettingFieldKind, PluginSettingsSection, PluginSetupContext, PluginSetupError,
    PluginStyleAsset, PluginViewDefinition, RegistryError, SettingsRegistry, ViewMount,
    ViewRegistry,
};
pub use runtime::{
    authorize_bridge_request, execute_command, CommandExecution, CommandOutcome, RuntimeError,
};
pub use storage::{
    DiskExtensionStore, InMemoryExtensionStore, PluginStorage, StorageBlob, StorageBlobMetadata,
    StorageBlobSyncClass, StorageError,
};

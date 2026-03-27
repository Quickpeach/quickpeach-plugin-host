use crate::manifest::{
    ExtensionCapability, ExtensionCommandV1, ExtensionManifestV1, ExtensionPermission,
    ExtensionPreferenceV1, ExtensionViewV1, CURRENT_SCHEMA_VERSION,
};
use crate::platform::PluginPlatformManifest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpstreamSourceKind {
    Raycast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityLevel {
    SupportedNow,
    PartialLater,
    UnsupportedInitially,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityMatrix {
    pub launcher_commands: CompatibilityLevel,
    pub simple_views: CompatibilityLevel,
    pub clipboard: CompatibilityLevel,
    pub browser_open_url: CompatibilityLevel,
    pub ai_calls: CompatibilityLevel,
    pub file_workflows: CompatibilityLevel,
    pub provider_auth: CompatibilityLevel,
    pub browser_tabs: CompatibilityLevel,
    pub node_runtime_assumptions: CompatibilityLevel,
    pub direct_native_apis: CompatibilityLevel,
    pub arbitrary_package_execution: CompatibilityLevel,
}

impl Default for CompatibilityMatrix {
    fn default() -> Self {
        Self {
            launcher_commands: CompatibilityLevel::SupportedNow,
            simple_views: CompatibilityLevel::SupportedNow,
            clipboard: CompatibilityLevel::SupportedNow,
            browser_open_url: CompatibilityLevel::SupportedNow,
            ai_calls: CompatibilityLevel::SupportedNow,
            file_workflows: CompatibilityLevel::PartialLater,
            provider_auth: CompatibilityLevel::PartialLater,
            browser_tabs: CompatibilityLevel::PartialLater,
            node_runtime_assumptions: CompatibilityLevel::UnsupportedInitially,
            direct_native_apis: CompatibilityLevel::UnsupportedInitially,
            arbitrary_package_execution: CompatibilityLevel::UnsupportedInitially,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalExtensionDescriptor {
    pub source: UpstreamSourceKind,
    pub repo: String,
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
    #[serde(default)]
    pub needs_oauth: bool,
    #[serde(default)]
    pub needs_node_runtime: bool,
    #[serde(default)]
    pub uses_browser_tabs: bool,
    #[serde(default)]
    pub uses_direct_native_apis: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub manifest: ExtensionManifestV1,
    pub compatibility: CompatibilityMatrix,
    pub warnings: Vec<String>,
}

pub fn translate_external_repo(descriptor: ExternalExtensionDescriptor) -> ImportPreview {
    let mut compatibility = CompatibilityMatrix::default();
    let mut warnings = Vec::new();

    if descriptor.needs_oauth {
        compatibility.provider_auth = CompatibilityLevel::PartialLater;
        warnings.push("Provider auth/OAuth should be implemented by Peach host later.".to_string());
    }

    if descriptor.needs_node_runtime {
        compatibility.node_runtime_assumptions = CompatibilityLevel::UnsupportedInitially;
        warnings.push(
            "Direct Node runtime assumptions are not supported in Peach v1 imports.".to_string(),
        );
    }

    if descriptor.uses_browser_tabs {
        compatibility.browser_tabs = CompatibilityLevel::PartialLater;
        warnings.push(
            "Browser tab integration should stay platform-gated in Peach imports.".to_string(),
        );
    }

    if descriptor.uses_direct_native_apis {
        compatibility.direct_native_apis = CompatibilityLevel::UnsupportedInitially;
        warnings.push(
            "Direct native APIs must be translated into Peach host bridge calls.".to_string(),
        );
    }

    let manifest = ExtensionManifestV1 {
        schema_version: CURRENT_SCHEMA_VERSION,
        id: descriptor.id,
        name: descriptor.name,
        description: descriptor.description,
        version: descriptor.version,
        author: descriptor.author,
        icon: descriptor.icon,
        commands: descriptor.commands,
        view: descriptor.view,
        permissions: descriptor.permissions,
        capabilities: descriptor.capabilities,
        preferences: descriptor.preferences,
        platform: descriptor.platform,
    };

    ImportPreview {
        manifest,
        compatibility,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ExtensionCommandActionV1, ExtensionCommandV1};

    #[test]
    fn translates_external_repo_metadata_into_a_peach_preview() {
        let preview = translate_external_repo(ExternalExtensionDescriptor {
            source: UpstreamSourceKind::Raycast,
            repo: "raycast/example".to_string(),
            id: "raycast-search".to_string(),
            name: "Raycast Search".to_string(),
            description: "Searches the web".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Raycast".to_string()),
            icon: None,
            commands: vec![ExtensionCommandV1 {
                id: "search".to_string(),
                name: "Search".to_string(),
                description: "Search the web".to_string(),
                keywords: vec![],
                action: ExtensionCommandActionV1::SearchWeb {
                    query: "peach note".to_string(),
                    provider: crate::bridge::SearchProvider::Default,
                },
            }],
            view: None,
            permissions: BTreeSet::from([ExtensionPermission::BrowserSearch]),
            capabilities: BTreeSet::from([
                ExtensionCapability::LauncherCommands,
                ExtensionCapability::Browser,
            ]),
            preferences: vec![],
            platform: None,
            needs_oauth: true,
            needs_node_runtime: true,
            uses_browser_tabs: false,
            uses_direct_native_apis: false,
        });

        assert_eq!(preview.manifest.id, "raycast-search");
        assert!(preview.warnings.len() >= 2);
        assert_eq!(
            preview.compatibility.node_runtime_assumptions,
            CompatibilityLevel::UnsupportedInitially
        );
    }
}

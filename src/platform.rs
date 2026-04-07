use crate::storage::PluginStorage;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::collections::BTreeMap;
use std::path::{Component, Path};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginStyleAsset {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub media: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginLauncherContract {
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub view_id: Option<String>,
    #[serde(default)]
    pub dynamic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginProviderDescriptor {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub requires_auth: bool,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardPosition {
    Main,
    #[default]
    Side,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWidgetDefinition {
    pub plugin_id: String,
    pub slot: String,
    pub title: String,
    pub component: String,
    #[serde(default)]
    pub position: DashboardPosition,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub min_height: Option<u16>,
    #[serde(default)]
    pub styles: Vec<PluginStyleAsset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PluginSettingFieldKind {
    #[default]
    String,
    Boolean,
    Number,
    Enum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PluginSettingField {
    String {
        key: String,
        label: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        placeholder: Option<String>,
        #[serde(default)]
        default: Option<String>,
        #[serde(default)]
        secret: bool,
    },
    Boolean {
        key: String,
        label: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        default: bool,
    },
    Number {
        key: String,
        label: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        default: Option<f64>,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
        #[serde(default)]
        step: Option<f64>,
    },
    Enum {
        key: String,
        label: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        default: Option<String>,
        #[serde(default)]
        options: Vec<String>,
    },
}

impl PluginSettingField {
    pub fn key(&self) -> &str {
        match self {
            Self::String { key, .. }
            | Self::Boolean { key, .. }
            | Self::Number { key, .. }
            | Self::Enum { key, .. } => key,
        }
    }

    pub fn default_value(&self) -> Option<Value> {
        match self {
            Self::String { default, .. } | Self::Enum { default, .. } => {
                default.clone().map(Value::String)
            }
            Self::Boolean { default, .. } => Some(Value::Bool(*default)),
            Self::Number { default, .. } => default.and_then(Number::from_f64).map(Value::Number),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSettingsSection {
    pub plugin_id: String,
    pub section_id: String,
    pub title: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub fields: Vec<PluginSettingField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ViewMount {
    #[default]
    Workspace,
    Launcher,
    DashboardWidget,
    Overlay,
    SettingsSection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginViewDefinition {
    pub plugin_id: String,
    pub view_id: String,
    pub entry: String,
    #[serde(default)]
    pub export_name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub mount: ViewMount,
    #[serde(default)]
    pub styles: Vec<PluginStyleAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayFamilyRegistration {
    pub plugin_id: String,
    pub namespace: String,
    pub display_name: String,
    #[serde(default)]
    pub default_view: Option<String>,
    #[serde(default)]
    pub multi_instance: bool,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)]
    pub focus_priority: u32,
    #[serde(default)]
    pub plain_transparency: bool,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginPlatformManifest {
    #[serde(default)]
    pub launcher: Option<PluginLauncherContract>,
    #[serde(default)]
    pub providers: Vec<PluginProviderDescriptor>,
    #[serde(default)]
    pub views: Vec<PluginViewDefinition>,
    #[serde(default)]
    pub dashboard_widgets: Vec<DashboardWidgetDefinition>,
    #[serde(default)]
    pub settings_sections: Vec<PluginSettingsSection>,
    #[serde(default)]
    pub overlay_families: Vec<OverlayFamilyRegistration>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("dashboard widget '{0}' is already registered")]
    DuplicateDashboardWidget(String),
    #[error("settings section '{0}' is already registered")]
    DuplicateSettingsSection(String),
    #[error("settings section '{section}' already contains field '{field}'")]
    DuplicateSettingsField { section: String, field: String },
    #[error("plugin view '{0}' is already registered")]
    DuplicateView(String),
    #[error("overlay namespace '{0}' is already registered")]
    DuplicateOverlayNamespace(String),
    #[error("asset path '{0}' must be a safe relative path inside the plugin package")]
    InvalidAssetPath(String),
    #[error("view entry '{0}' must be a safe relative path inside the plugin package")]
    InvalidViewEntry(String),
}

#[derive(Debug, Default, Clone)]
pub struct DashboardRegistry {
    widgets: BTreeMap<String, DashboardWidgetDefinition>,
}

impl DashboardRegistry {
    pub fn register(&mut self, widget: DashboardWidgetDefinition) -> Result<(), RegistryError> {
        let key = format!("{}::{}", widget.plugin_id, widget.slot);
        if self.widgets.contains_key(&key) {
            return Err(RegistryError::DuplicateDashboardWidget(key));
        }

        for style in &widget.styles {
            ensure_safe_relative_path(&style.path)
                .map_err(|_| RegistryError::InvalidAssetPath(style.path.clone()))?;
        }

        self.widgets.insert(key, widget);
        Ok(())
    }

    pub fn all(&self) -> Vec<&DashboardWidgetDefinition> {
        let mut widgets = self.widgets.values().collect::<Vec<_>>();
        widgets.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then_with(|| right.priority.cmp(&left.priority))
                .then_with(|| left.title.cmp(&right.title))
                .then_with(|| left.plugin_id.cmp(&right.plugin_id))
                .then_with(|| left.slot.cmp(&right.slot))
        });
        widgets
    }
}

#[derive(Debug, Default, Clone)]
pub struct SettingsRegistry {
    sections: BTreeMap<String, PluginSettingsSection>,
}

impl SettingsRegistry {
    pub fn register_section(
        &mut self,
        section: PluginSettingsSection,
    ) -> Result<(), RegistryError> {
        let key = format!("{}::{}", section.plugin_id, section.section_id);
        if self.sections.contains_key(&key) {
            return Err(RegistryError::DuplicateSettingsSection(key));
        }

        let mut seen = BTreeMap::new();
        for field in &section.fields {
            if seen.insert(field.key().to_string(), ()).is_some() {
                return Err(RegistryError::DuplicateSettingsField {
                    section: section.section_id.clone(),
                    field: field.key().to_string(),
                });
            }
        }

        self.sections.insert(key, section);
        Ok(())
    }

    pub fn all(&self) -> Vec<&PluginSettingsSection> {
        self.sections.values().collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ViewRegistry {
    views: BTreeMap<String, PluginViewDefinition>,
}

impl ViewRegistry {
    pub fn register(&mut self, view: PluginViewDefinition) -> Result<(), RegistryError> {
        let key = format!("{}::{}", view.plugin_id, view.view_id);
        if self.views.contains_key(&key) {
            return Err(RegistryError::DuplicateView(key));
        }

        ensure_safe_relative_path(&view.entry)
            .map_err(|_| RegistryError::InvalidViewEntry(view.entry.clone()))?;
        for style in &view.styles {
            ensure_safe_relative_path(&style.path)
                .map_err(|_| RegistryError::InvalidAssetPath(style.path.clone()))?;
        }

        self.views.insert(key, view);
        Ok(())
    }

    pub fn all(&self) -> Vec<&PluginViewDefinition> {
        self.views.values().collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct OverlayRegistryPlan {
    overlays: BTreeMap<String, OverlayFamilyRegistration>,
}

impl OverlayRegistryPlan {
    pub fn register(&mut self, overlay: OverlayFamilyRegistration) -> Result<(), RegistryError> {
        if self.overlays.contains_key(&overlay.namespace) {
            return Err(RegistryError::DuplicateOverlayNamespace(
                overlay.namespace.clone(),
            ));
        }

        self.overlays.insert(overlay.namespace.clone(), overlay);
        Ok(())
    }

    pub fn all(&self) -> Vec<&OverlayFamilyRegistration> {
        self.overlays.values().collect()
    }
}

pub struct PluginSetupContext<'a> {
    pub plugin_id: &'a str,
    pub storage: &'a mut dyn PluginStorage,
    pub dashboard: &'a mut DashboardRegistry,
    pub settings: &'a mut SettingsRegistry,
    pub views: &'a mut ViewRegistry,
    pub overlays: &'a mut OverlayRegistryPlan,
}

#[derive(Debug, Error)]
pub enum PluginSetupError {
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error("{0}")]
    Message(String),
}

pub trait PluginHost: Send + Sync {
    fn descriptor(&self) -> PluginDescriptor;

    fn setup(&self, ctx: &mut PluginSetupContext<'_>) -> Result<(), PluginSetupError>;

    fn teardown(&self, _ctx: &PluginSetupContext<'_>) -> Result<(), PluginSetupError> {
        Ok(())
    }
}

fn ensure_safe_relative_path(path: &str) -> Result<(), ()> {
    let candidate = Path::new(path);
    if candidate.is_absolute() || path.trim().is_empty() {
        return Err(());
    }

    for component in candidate.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err(()),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_dashboard_widgets_by_position_and_priority() {
        let mut registry = DashboardRegistry::default();
        registry
            .register(DashboardWidgetDefinition {
                plugin_id: "music".into(),
                slot: "now-playing".into(),
                title: "Now Playing".into(),
                component: "MusicNowPlaying".into(),
                position: DashboardPosition::Side,
                priority: 50,
                min_height: Some(80),
                styles: vec![],
            })
            .unwrap();
        registry
            .register(DashboardWidgetDefinition {
                plugin_id: "calendar-plus".into(),
                slot: "day-view".into(),
                title: "Day View".into(),
                component: "DayView".into(),
                position: DashboardPosition::Main,
                priority: 10,
                min_height: None,
                styles: vec![],
            })
            .unwrap();

        let widgets = registry.all();
        assert_eq!(widgets[0].slot, "day-view");
        assert_eq!(widgets[1].slot, "now-playing");
    }

    #[test]
    fn rejects_duplicate_settings_fields() {
        let mut registry = SettingsRegistry::default();
        let error = registry
            .register_section(PluginSettingsSection {
                plugin_id: "music".into(),
                section_id: "music".into(),
                title: "Music".into(),
                icon: Some("music".into()),
                fields: vec![
                    PluginSettingField::String {
                        key: "provider".into(),
                        label: "Provider".into(),
                        description: None,
                        placeholder: None,
                        default: None,
                        secret: false,
                    },
                    PluginSettingField::Enum {
                        key: "provider".into(),
                        label: "Provider".into(),
                        description: None,
                        default: None,
                        options: vec!["spotify".into()],
                    },
                ],
            })
            .expect_err("duplicate field should fail");

        assert!(matches!(
            error,
            RegistryError::DuplicateSettingsField { section, field }
                if section == "music" && field == "provider"
        ));
    }

    #[test]
    fn rejects_invalid_view_and_style_paths() {
        let mut views = ViewRegistry::default();
        let error = views
            .register(PluginViewDefinition {
                plugin_id: "music".into(),
                view_id: "hub".into(),
                entry: "../outside.js".into(),
                export_name: None,
                title: Some("Hub".into()),
                mount: ViewMount::Workspace,
                styles: vec![],
            })
            .expect_err("unsafe path should fail");

        assert!(matches!(error, RegistryError::InvalidViewEntry(path) if path == "../outside.js"));
    }

    #[test]
    fn rejects_duplicate_overlay_namespaces() {
        let mut overlays = OverlayRegistryPlan::default();
        let overlay = OverlayFamilyRegistration {
            plugin_id: "music".into(),
            namespace: "music".into(),
            display_name: "Music".into(),
            default_view: Some("music-hub".into()),
            multi_instance: false,
            always_on_top: false,
            focus_priority: 12,
            plain_transparency: false,
            width: 520,
            height: 600,
        };
        overlays.register(overlay.clone()).unwrap();
        let error = overlays
            .register(overlay)
            .expect_err("duplicate namespace should fail");

        assert!(matches!(
            error,
            RegistryError::DuplicateOverlayNamespace(namespace) if namespace == "music"
        ));
    }
}

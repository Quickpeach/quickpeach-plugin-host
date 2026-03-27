use crate::bridge::{BridgeRequest, HostEffect};
use crate::discovery::LoadedExtension;
use crate::manifest::{ExtensionCommandActionV1, ExtensionManifestV1, ExtensionPermission};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CommandOutcome {
    Immediate {
        #[serde(default)]
        message: Option<String>,
    },
    Items {
        items: Vec<crate::bridge::LauncherListItem>,
    },
    AsyncAccepted {
        task_id: String,
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecution {
    pub extension_id: String,
    pub command_id: String,
    pub outcome: CommandOutcome,
    pub effects: Vec<HostEffect>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RuntimeError {
    #[error("command '{0}' not found")]
    CommandNotFound(String),
    #[error("permission '{permission:?}' denied for extension '{extension_id}'")]
    PermissionDenied {
        extension_id: String,
        permission: ExtensionPermission,
    },
}

pub fn execute_command(
    extension: &LoadedExtension,
    command_id: &str,
) -> Result<CommandExecution, RuntimeError> {
    let command = extension
        .manifest
        .commands
        .iter()
        .find(|command| command.id == command_id)
        .ok_or_else(|| RuntimeError::CommandNotFound(command_id.to_string()))?;

    ensure_permissions(
        &extension.manifest,
        &extension.manifest.id,
        command.action.required_permissions(),
    )?;

    let (outcome, effects) = match &command.action {
        ExtensionCommandActionV1::OpenUrl { url } => (
            CommandOutcome::Immediate {
                message: Some(format!("Opening {}", url)),
            },
            vec![HostEffect::Bridge {
                request: BridgeRequest::BrowserOpen { url: url.clone() },
            }],
        ),
        ExtensionCommandActionV1::SearchWeb { query, provider } => (
            CommandOutcome::Immediate {
                message: Some(format!("Searching {}", query)),
            },
            vec![HostEffect::Bridge {
                request: BridgeRequest::BrowserSearch {
                    query: query.clone(),
                    provider: *provider,
                },
            }],
        ),
        ExtensionCommandActionV1::ClipboardWrite { text } => (
            CommandOutcome::Immediate {
                message: Some("Clipboard updated".to_string()),
            },
            vec![HostEffect::Bridge {
                request: BridgeRequest::ClipboardWrite { text: text.clone() },
            }],
        ),
        ExtensionCommandActionV1::EmitEvent { event, payload } => (
            CommandOutcome::Immediate {
                message: Some(format!("Event {}", event)),
            },
            vec![HostEffect::Bridge {
                request: BridgeRequest::EmitEvent {
                    event: event.clone(),
                    payload: payload.clone(),
                },
            }],
        ),
        ExtensionCommandActionV1::AiPrompt { prompt, model } => (
            CommandOutcome::Immediate {
                message: Some("AI request queued".to_string()),
            },
            vec![HostEffect::Bridge {
                request: BridgeRequest::AiInvoke {
                    prompt: prompt.clone(),
                    model: model.clone(),
                },
            }],
        ),
        ExtensionCommandActionV1::SearchFiles {
            query,
            roots,
            extensions,
            include_hidden,
            max_depth,
            limit,
        } => (
            CommandOutcome::Immediate {
                message: Some("Searching files".to_string()),
            },
            vec![HostEffect::Bridge {
                request: BridgeRequest::SearchFiles {
                    query: query.clone(),
                    roots: roots.clone(),
                    extensions: extensions.clone(),
                    include_hidden: *include_hidden,
                    max_depth: *max_depth,
                    limit: *limit,
                },
            }],
        ),
        ExtensionCommandActionV1::CalendarSync { source_ids } => (
            CommandOutcome::Immediate {
                message: Some("Syncing calendars".to_string()),
            },
            vec![HostEffect::Bridge {
                request: BridgeRequest::CalendarSync {
                    source_ids: source_ids.clone(),
                },
            }],
        ),
        ExtensionCommandActionV1::CalendarListEvents {
            source_ids,
            from,
            to,
            query,
            limit,
        } => (
            CommandOutcome::Immediate {
                message: Some("Loading calendar events".to_string()),
            },
            vec![HostEffect::Bridge {
                request: BridgeRequest::CalendarListEvents {
                    source_ids: source_ids.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    query: query.clone(),
                    limit: *limit,
                },
            }],
        ),
        ExtensionCommandActionV1::ReturnItems { items } => (
            CommandOutcome::Items {
                items: items.clone(),
            },
            Vec::new(),
        ),
        ExtensionCommandActionV1::AsyncTask { task_id, label } => (
            CommandOutcome::AsyncAccepted {
                task_id: task_id.clone(),
                label: label.clone(),
            },
            Vec::new(),
        ),
    };

    Ok(CommandExecution {
        extension_id: extension.manifest.id.clone(),
        command_id: command.id.clone(),
        outcome,
        effects,
    })
}

pub fn authorize_bridge_request(
    extension: &LoadedExtension,
    request: BridgeRequest,
) -> Result<HostEffect, RuntimeError> {
    if let Some(permission) = request.required_permission() {
        ensure_permission(&extension.manifest, &extension.manifest.id, permission)?;
    }

    Ok(HostEffect::Bridge { request })
}

fn ensure_permissions<I>(
    manifest: &ExtensionManifestV1,
    extension_id: &str,
    permissions: I,
) -> Result<(), RuntimeError>
where
    I: IntoIterator<Item = ExtensionPermission>,
{
    for permission in permissions {
        ensure_permission(manifest, extension_id, permission)?;
    }
    Ok(())
}

fn ensure_permission(
    manifest: &ExtensionManifestV1,
    extension_id: &str,
    permission: ExtensionPermission,
) -> Result<(), RuntimeError> {
    if manifest.permissions.contains(&permission) {
        Ok(())
    } else {
        Err(RuntimeError::PermissionDenied {
            extension_id: extension_id.to_string(),
            permission,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{ExtensionSource, LoadedExtension};
    use crate::manifest::{
        ExtensionCapability, ExtensionCommandV1, ExtensionManifestV1, ExtensionPermission,
    };
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn extension() -> LoadedExtension {
        LoadedExtension {
            manifest: ExtensionManifestV1 {
                schema_version: 1,
                id: "browser-tools".to_string(),
                name: "Browser Tools".to_string(),
                description: "Search tools".to_string(),
                version: "0.1.0".to_string(),
                author: None,
                icon: None,
                commands: vec![ExtensionCommandV1 {
                    id: "search-web".to_string(),
                    name: "Search Web".to_string(),
                    description: "Searches the web".to_string(),
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
            },
            path: PathBuf::from("/tmp/browser-tools"),
            source: ExtensionSource::LocalDirectory,
            enabled: true,
        }
    }

    #[test]
    fn executes_allowed_commands() {
        let execution = execute_command(&extension(), "search-web").expect("command should run");
        assert_eq!(execution.effects.len(), 1);
    }

    #[test]
    fn denies_missing_bridge_permissions() {
        let mut extension = extension();
        extension.manifest.permissions.clear();

        let error = authorize_bridge_request(
            &extension,
            BridgeRequest::BrowserSearch {
                query: "hi".to_string(),
                provider: crate::bridge::SearchProvider::Default,
            },
        )
        .expect_err("permission should be denied");

        assert!(matches!(
            error,
            RuntimeError::PermissionDenied {
                extension_id,
                permission: ExtensionPermission::BrowserSearch
            } if extension_id == "browser-tools"
        ));
    }
}

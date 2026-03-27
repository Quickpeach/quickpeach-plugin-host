# quickpeach-plugin-host

The plugin runtime foundation for QuickPeach. This Rust crate defines the contracts that QuickPeach and its plugins agree on -- manifest formats, permission checks, storage, and registration APIs.

It doesn't run plugin code itself. Instead, it's the layer between QuickPeach's app shell and the plugins that extend it, providing:

- A versioned manifest format so plugins can describe what they need
- Local plugin discovery and validation
- Permission-checked command execution
- Typed request/response models for host capabilities (clipboard, browser, AI, storage, notes, preferences, calendar, launcher results, file search)
- Per-plugin key/value storage
- Registries for dashboard widgets, settings panels, overlay windows, and plugin views
- Lifecycle hooks so plugins can set themselves up at load time
- An importer that previews how public extension repos would translate into QuickPeach manifests

## Why this crate exists

QuickPeach's plugin system is split into two layers. This crate is the open, portable contract layer. The QuickPeach app shell handles product-specific concerns (UI, services, marketplace) on top of it.

This separation means:

- **QuickPeach** exposes well-typed plugin APIs without giving plugins raw shell access.
- **Plugin authors** target stable manifest, storage, and registration contracts that won't break between releases.
- **The contracts stay inspectable** while the app shell and backend can remain proprietary.

## Using the crate

This is a Rust library crate, so consumers normally add it as a dependency with
`cargo add`, not `cargo install`.

### Local workspace

```toml
[dependencies]
peachnote-plugin-host = { path = "../peachnote-plugin-host" }
```

Current packaging status:

- local path usage works today
- the crate is packageable as a library
- publishing still requires the shared `peachnote-vault-core` dependency to be
  available from the source you publish against, typically crates.io or a shared
  git workspace

## Modules at a glance

| Module | What it does |
|---|---|
| `manifest` | Schema, permissions, preferences, views, and validation for plugin manifests |
| `runtime` | Plans command execution after checking permissions |
| `bridge` | Typed request/response models between plugins and the host |
| `storage` | `PluginStorage` trait plus in-memory and disk-backed implementations |
| `platform` | `PluginHost` trait, setup context, and registries for dashboards, settings, views, and overlays |
| `discovery` | Loads unpacked extensions from a local directory |
| `importer` | Generates translation previews from public extension repos |

## Platform layer

The `platform` module is where plugin lifecycle and registration live. Plugins implement the `PluginHost` trait and use the setup context to register:

- Dashboard widgets
- Settings sections with typed fields
- Plugin views (React components the host will render)
- Overlay window families
- Lifecycle hooks via `on_setup()`

This crate only defines the data contracts. It does not decide how QuickPeach actually renders React views or injects CSS -- that's the app shell's job.

## Storage layer

Two storage backends are included:

- **`InMemoryExtensionStore`** -- fast, process-local storage for tests.
- **`DiskExtensionStore`** -- durable, JSON-backed storage namespaced by extension id.

This gives plugins persistent storage without tying QuickPeach to a specific database engine.

## Trust model

Extensions come from one of two sources:

- **`LocalDirectory`** -- unpacked folders on disk (for development or sideloading).
- **`NativeRegistry`** -- managed by QuickPeach, where it can add signing, pinned hashes, and curated installs.

The runtime is conservative by default:

- Every command and bridge request is checked against the plugin's declared permissions.
- The manifest/runtime contract never exposes arbitrary shell execution.
- Privileged native behavior stays in QuickPeach's app shell, not in unpacked local plugins.

QuickPeach decides the final trust policy. This crate just enforces the permission checks.

## What QuickPeach still needs to provide

This crate handles contracts, not logistics. QuickPeach's app shell is responsible for:

- Package discovery and installation UI
- Signature verification and provenance checks
- Digest checking for downloaded bundles
- Final permission review UI
- Actually rendering plugin views and widgets
- Distinguishing community packages from first-party ones

## Example manifest

A minimal plugin manifest looks like this:

```json
{
  "schemaVersion": 1,
  "id": "clipboard-tools",
  "name": "Clipboard Tools",
  "description": "Quick clipboard helpers",
  "version": "0.1.0",
  "capabilities": ["launcher-commands", "clipboard"],
  "permissions": ["clipboard-write"],
  "commands": [
    {
      "id": "copy-hello",
      "name": "Copy Hello",
      "description": "Copies a fixed string",
      "action": {
        "type": "clipboard-write",
        "text": "Hello from Peach Note"
      }
    }
  ]
}
```

## Example plugin setup

Here's what a plugin looks like in Rust. It registers an overlay window, a view, and a dashboard widget:

```rust
use peachnote_plugin_host::{
    DashboardPosition, DashboardRegistry, DashboardWidgetDefinition, OverlayFamilyRegistration,
    PluginDescriptor, PluginHost, PluginSetupContext, PluginStyleAsset, PluginViewDefinition,
    SettingsRegistry, ViewMount,
};

struct MusicPlugin;

impl PluginHost for MusicPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "peach-music".into(),
            name: "Peach Music".into(),
            version: "0.1.0".into(),
        }
    }

    fn on_setup(&self, ctx: &mut PluginSetupContext<'_>) -> Result<(), peachnote_plugin_host::PluginSetupError> {
        // Register a 520x600 overlay window for the music UI
        ctx.overlays.register(OverlayFamilyRegistration {
            plugin_id: "peach-music".into(),
            namespace: "music".into(),
            display_name: "Music".into(),
            default_view: Some("music-hub".into()),
            multi_instance: false,
            always_on_top: false,
            focus_priority: 12,
            plain_transparency: false,
            width: 520,
            height: 600,
        })?;

        // Register the React view that renders inside the overlay
        ctx.views.register(PluginViewDefinition {
            plugin_id: "peach-music".into(),
            view_id: "music-hub".into(),
            entry: "dist/music-hub.js".into(),
            export_name: Some("MusicHub".into()),
            title: Some("Music".into()),
            mount: ViewMount::Overlay,
            styles: vec![PluginStyleAsset {
                id: "music".into(),
                path: "dist/music.css".into(),
                media: None,
            }],
        })?;

        // Add a "Now Playing" widget to the dashboard sidebar
        ctx.dashboard.register(DashboardWidgetDefinition {
            plugin_id: "peach-music".into(),
            slot: "now-playing".into(),
            title: "Now Playing".into(),
            component: "MusicNowPlaying".into(),
            position: DashboardPosition::Side,
            priority: 50,
            min_height: Some(88),
            styles: Vec::new(),
        })?;

        Ok(())
    }
}
```

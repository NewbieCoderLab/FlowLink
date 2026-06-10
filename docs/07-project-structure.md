# Project Directory Structure

## 1. Recommended Stack

- Tauri 2.
- Rust backend.
- TypeScript frontend.
- Vite for frontend build.
- Plain CSS or a small component system for V1.

Frontend framework:

- React is acceptable if team is already comfortable with it.
- Svelte is also a good fit for Tauri.
- For lowest complexity, choose React + TypeScript only if state and components justify it.

This document assumes React + TypeScript because it is widely supported by AI coding agents.

## 2. Directory Tree

```text
mac22win/
  README.md
  package.json
  pnpm-lock.yaml
  index.html
  vite.config.ts
  tsconfig.json
  docs/
    README.md
    01-prd.md
    02-technical-architecture.md
    03-module-breakdown.md
    04-data-structures.md
    05-network-protocol.md
    06-local-storage.md
    07-project-structure.md
    08-mvp-roadmap.md
    09-risk-assessment.md
    10-v2-v3-evolution.md
  src/
    main.tsx
    app/
      App.tsx
      tauri.ts
      types.ts
    components/
      DeviceList.tsx
      DeviceCard.tsx
      LayoutEditor.tsx
      PermissionPanel.tsx
      PairingDialog.tsx
      StatusBar.tsx
      DiagnosticsPanel.tsx
    hooks/
      useAppStatus.ts
      useDiscoveredDevices.ts
      useSession.ts
      useTauriEvent.ts
    styles/
      base.css
      app.css
  src-tauri/
    Cargo.toml
    build.rs
    tauri.conf.json
    capabilities/
      default.json
    icons/
    src/
      main.rs
      lib.rs
      app/
        mod.rs
        context.rs
        lifecycle.rs
      config/
        mod.rs
        defaults.rs
        validate.rs
      discovery/
        mod.rs
        mdns.rs
        udp.rs
        cache.rs
      identity/
        mod.rs
        keys.rs
        hostname.rs
      input/
        mod.rs
        types.rs
        edge.rs
        macos.rs
        windows.rs
        noop.rs
      network/
        mod.rs
        listener.rs
        connector.rs
        framing.rs
        heartbeat.rs
        reconnect.rs
      pairing/
        mod.rs
        code.rs
        flow.rs
      protocol/
        mod.rs
        frame.rs
        messages.rs
        version.rs
      session/
        mod.rs
        state.rs
        controller.rs
      storage/
        mod.rs
        files.rs
        secret.rs
        migrations.rs
      telemetry/
        mod.rs
        logging.rs
        metrics.rs
      ui_api/
        mod.rs
        commands.rs
        events.rs
        models.rs
      platform/
        mod.rs
        macos_permissions.rs
        windows_permissions.rs
    tests/
      protocol_tests.rs
      storage_tests.rs
      pairing_tests.rs
```

## 3. Rust Module Rules

### 3.1 Keep UI out of hot path

High-frequency mouse events must stay in Rust:

```text
input hook -> session -> network
```

Do not send every mouse event to TypeScript.

UI receives only summarized status:

- Current control owner.
- Connection state.
- Last heartbeat RTT.
- Dropped/coalesced move count.

### 3.2 Platform-specific code

Use conditional compilation:

```rust
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;
```

Provide `noop` implementation for unsupported platforms so the frontend can still develop on other systems if needed.

### 3.3 Error Handling

Use `thiserror` for internal typed errors:

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("permission missing: {0}")]
    PermissionMissing(String),
    #[error("network error: {0}")]
    Network(String),
}
```

Convert to UI errors:

```rust
pub struct UiError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}
```

### 3.4 Async Runtime

Use Tokio:

- Discovery sockets.
- TCP listener/client.
- Heartbeats.
- Reconnect loops.
- Storage can be synchronous if small and guarded, but async is fine.

## 4. Suggested Rust Dependencies

Core:

```toml
tauri = "2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4", "serde"] }
bytes = "1"
```

Protocol:

```toml
bincode = "1"
sha2 = "0.10"
rand = "0.8"
```

Discovery candidates:

```toml
mdns-sd = "0.13"
```

Windows:

```toml
windows = { version = "0.58", features = [
  "Win32_UI_Input_KeyboardAndMouse",
  "Win32_UI_WindowsAndMessaging",
  "Win32_Foundation",
  "Win32_Graphics_Gdi"
] }
```

macOS:

```toml
core-graphics = "0.24"
core-foundation = "0.10"
objc2 = "0.5"
```

Dependency versions should be checked when implementation starts.

## 5. Frontend Views

### 5.1 Main Window

Sections:

- Local device header.
- Permission panel.
- Discovered/paired devices list.
- Layout editor.
- Connection controls.
- Status bar.

States:

- No permission.
- No device discovered.
- Devices discovered.
- Pairing in progress.
- Connected.
- Controlling remote.
- Disconnected/reconnecting.

### 5.2 Pairing Dialog

Content:

- Remote device name.
- Remote OS and IP.
- Six-digit pairing code.
- Confirm and reject buttons.
- Expiry countdown.

### 5.3 Layout Editor

UI:

- Two rectangles representing local and remote.
- Segmented control: `Left`, `Right`, `Top`, `Bottom`.
- Active edge highlighted.
- Save/apply button.

### 5.4 Diagnostics Panel

Fields:

- Protocol version.
- Local port.
- Last heartbeat RTT.
- Reconnect attempts.
- Discovery source.
- Permission states.
- Recent errors.

## 6. Build Commands

Suggested scripts:

```json
{
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "typecheck": "tsc --noEmit",
    "test:rust": "cargo test --manifest-path src-tauri/Cargo.toml",
    "lint:rust": "cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings",
    "fmt:rust": "cargo fmt --manifest-path src-tauri/Cargo.toml",
    "check": "pnpm typecheck && pnpm test:rust && pnpm lint:rust"
  }
}
```

## 7. Packaging Notes

macOS:

- Configure bundle identifier.
- Add usage descriptions if APIs require plist strings.
- Test permission prompts on a signed build, not only dev mode.
- Notarization is not required for internal MVP but should be planned.

Windows:

- WebView2 runtime requirement should be documented or bundled according to Tauri guidance.
- Installer should add firewall prompt guidance if incoming TCP is blocked.
- Code signing can wait for internal MVP but affects trust prompts.


# Local Storage Design

## 1. Storage Goals

- Persist identity, trusted peers, layout, and network settings.
- Survive app restart.
- Be easy to inspect during MVP development.
- Avoid corrupting config on crash.
- Keep private key handling isolated for later hardening.

## 2. Storage Location

Use Tauri application config/data directories.

Logical paths:

```text
<app_config_dir>/
  config.json
  identity.json
  trusted_peers.json
  layouts.json
  logs/
    app.log
```

Platform examples:

- macOS: `~/Library/Application Support/<bundle-id>/`
- Windows: `%APPDATA%\<bundle-id>\`

Bundle ID placeholder:

```text
com.mac22win.app
```

## 3. File Responsibilities

### 3.1 `identity.json`

Contains:

- Local stable device ID.
- Local public key.
- Private key reference.
- Device name override.
- Created timestamp.

Example:

```json
{
  "schema_version": 1,
  "device_id": "7fd37b26-caf6-43db-b6c1-16d9b48fd2d2",
  "device_name": "Alices MacBook Pro",
  "os": "macos",
  "arch": "aarch64",
  "app_version": "0.1.0",
  "protocol_version": 1,
  "public_key": "base64...",
  "private_key_ref": {
    "type": "file_encrypted",
    "path": "identity.key"
  },
  "created_at_ms": 1780000000000
}
```

### 3.2 `trusted_peers.json`

Contains:

- Paired devices.
- Public keys.
- Last known addresses.
- Trust state.

Example:

```json
{
  "schema_version": 1,
  "peers": [
    {
      "peer_id": "b2f8d4e8-e2aa-4888-a1ce-0af342bcb03b",
      "device_name": "Office Windows PC",
      "os": "windows",
      "arch": "x86_64",
      "public_key": "base64...",
      "last_known_addresses": ["192.168.1.42:42424"],
      "last_seen_ms": 1780000005000,
      "paired_at_ms": 1780000001000,
      "app_version_at_pairing": "0.1.0",
      "protocol_version": 1,
      "trust_state": "trusted"
    }
  ]
}
```

### 3.3 `layouts.json`

Contains:

- One layout config per peer.

Example:

```json
{
  "schema_version": 1,
  "layouts": [
    {
      "peer_id": "b2f8d4e8-e2aa-4888-a1ce-0af342bcb03b",
      "direction": "right",
      "edge_thickness_px": 1,
      "corner_guard_px": 32,
      "enabled": true,
      "updated_at_ms": 1780000006000
    }
  ]
}
```

### 3.4 `config.json`

Contains:

- Network defaults.
- Discovery defaults.
- UI preferences.

Example:

```json
{
  "schema_version": 1,
  "network": {
    "listen_port": 42424,
    "connect_timeout_ms": 3000,
    "heartbeat_interval_ms": 1000,
    "heartbeat_timeout_ms": 3000,
    "reconnect_min_delay_ms": 500,
    "reconnect_max_delay_ms": 10000
  },
  "discovery": {
    "mdns_enabled": true,
    "udp_broadcast_enabled": true,
    "udp_port": 42425,
    "announce_interval_ms": 1500,
    "stale_after_ms": 10000
  },
  "ui": {
    "start_minimized": false,
    "show_diagnostics": false,
    "last_selected_peer_id": null
  },
  "updated_at_ms": 1780000006000
}
```

## 4. Atomic Write Policy

For every file write:

1. Serialize to bytes.
2. Write to `<file>.tmp`.
3. Flush file.
4. Rename `<file>.tmp` to `<file>`.

On startup:

- If main file parses successfully, use it.
- If main file fails and tmp file parses successfully, recover tmp.
- If both fail, move corrupt files to backup names and use defaults.

Corrupt backup naming:

```text
config.json.corrupt.1780000000000
```

## 5. Schema Versioning

Every persisted file must include:

```json
{
  "schema_version": 1
}
```

Migration rules:

- V1 only needs schema `1`.
- If schema is newer than app supports, do not overwrite automatically.
- Show error and keep app running with limited functionality.

## 6. Secret Storage

Recommended:

- macOS: Keychain for private key.
- Windows: Credential Manager or DPAPI-protected file.

MVP fallback:

- Store private key file in app data directory.
- Set restrictive file permissions.
- Make storage abstraction explicit so V2 can move secrets without changing pairing/session code.

Rust abstraction:

```rust
pub trait SecretStore {
    fn load_private_key(&self) -> Result<Vec<u8>>;
    fn save_private_key(&self, key: &[u8]) -> Result<PrivateKeyRef>;
}
```

## 7. Logs

Location:

```text
<app_config_dir>/logs/app.log
```

Rotation:

- Max file size: `5MB`.
- Keep `3` rotated files.

Privacy:

- Do not log raw pairing private keys.
- Do not log full auth secrets.
- Do not log high-frequency mouse coordinates by default.
- It is acceptable to log peer device name, OS, IP address, state transitions, and error codes for local diagnostics.

## 8. UI Storage Behavior

UI must not write storage directly.

All updates go through Tauri commands:

- `save_layout`.
- `forget_peer`.
- `rename_local_device`.
- `update_network_config`.

The Rust core validates and persists.

## 9. Manual Reset

Support reset actions:

- Forget selected peer.
- Reset all pairings.
- Reset layout.
- Reset app config.

Do not delete identity automatically when resetting pairings. Device identity reset should be a separate advanced action because it invalidates existing trust relationships.


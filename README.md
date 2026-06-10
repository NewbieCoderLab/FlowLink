# FlowLink

FlowLink is a Tauri + Rust + React MVP scaffold for cross-device mouse handoff on macOS and Windows.

This repository is implemented from the product and architecture documents in [`docs/`](./docs/), with the first delivery focused on:

- Tauri desktop shell with a React control panel
- Rust domain models for identity, layout, peers, session, and permissions
- Binary protocol framing and pairing-code utilities
- JSON storage with atomic writes and schema-aware defaults
- Session state management and edge-detection primitives

## Structure

- `src/`: React + TypeScript UI
- `src-tauri/`: Rust core and desktop shell
- `docs/`: PRD and technical design reference

## Next Steps

- Wire real mDNS discovery
- Add trusted TCP session handshake
- Implement platform-specific input capture/injection
- Integrate pairing confirmation and heartbeat events


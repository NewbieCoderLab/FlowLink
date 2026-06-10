# MVP Development Roadmap

## Phase 0: Spike And Feasibility

Goal:

- Prove both platforms can capture and inject mouse events.

Tasks:

- Create minimal Tauri app.
- Implement macOS permission checks.
- Implement macOS mouse event tap capture.
- Implement macOS mouse injection smoke test.
- Implement Windows `WH_MOUSE_LL` capture.
- Implement Windows `SendInput` injection smoke test.
- Measure local capture-to-inject latency inside one machine.

Exit criteria:

- macOS can observe move/down/up/wheel after permissions are granted.
- macOS can inject move/down/up/wheel into normal apps.
- Windows can observe move/down/up/wheel as normal user.
- Windows can inject move/down/up/wheel into normal apps.
- Known OS limitations are documented.

Suggested duration:

- 3 to 5 engineering days.

## Phase 1: Project Scaffold

Goal:

- Establish app structure, config, UI shell, and logging.

Tasks:

- Scaffold Tauri + TypeScript project.
- Add Rust modules from project structure.
- Add `tracing` logging.
- Add config and identity storage.
- Add basic UI with local device status.
- Add Tauri commands for app status and permission status.

Exit criteria:

- App starts on macOS and Windows.
- Device identity persists after restart.
- Permission status appears in UI.
- Logs are written locally.

Suggested duration:

- 2 to 4 engineering days.

## Phase 2: Discovery

Goal:

- Devices in the same LAN discover each other.

Tasks:

- Implement mDNS publish.
- Implement mDNS browse.
- Implement discovery cache.
- Implement stale peer removal.
- Add UDP broadcast fallback.
- Add discovered device UI.

Exit criteria:

- Peer appears within `3s` on normal LAN.
- Peer disappears or marks stale after `10s`.
- Device list shows name, OS, IP, app version.
- Self is filtered.

Suggested duration:

- 3 to 5 engineering days.

## Phase 3: Pairing

Goal:

- Users can safely pair two devices.

Tasks:

- Implement pairing request/response protocol.
- Generate six-digit pairing code.
- Add confirmation dialogs on both devices.
- Persist trusted peers.
- Add IP direct connect flow.
- Add reject/expiry handling.

Exit criteria:

- Pairing code flow succeeds.
- IP direct connect succeeds.
- Rejection does not create trust.
- Expired code cannot pair.
- Paired peer persists after restart.

Suggested duration:

- 4 to 7 engineering days.

## Phase 4: TCP Session And Protocol

Goal:

- Trusted peers maintain a low-latency bidirectional channel.

Tasks:

- Implement length-prefixed frame parser.
- Implement message encode/decode.
- Implement TCP listener/client.
- Enable `TCP_NODELAY`.
- Implement Hello validation.
- Implement heartbeat.
- Implement reconnect with backoff.
- Add protocol tests.

Exit criteria:

- Trusted peer connects automatically.
- Heartbeat RTT appears in diagnostics.
- Disconnect detected within `3s`.
- Reconnect works after app restart or network interruption.
- Parser tests pass.

Suggested duration:

- 4 to 7 engineering days.

## Phase 5: Layout Management

Goal:

- User can configure edge mapping.

Tasks:

- Implement layout storage.
- Implement layout editor UI.
- Implement screen topology read on both platforms.
- Implement edge detection.
- Add corner guard.

Exit criteria:

- Layout persists.
- Edge detection works for left/right/top/bottom.
- UI shows active layout and active edge.

Suggested duration:

- 2 to 4 engineering days.

## Phase 6: Mouse Handoff

Goal:

- Core MVP behavior works end to end.

Tasks:

- Connect local input capture to session controller.
- Send `ControlEnter` on edge trigger.
- Inject initial remote pointer position.
- Stream mouse move deltas.
- Send button and wheel events.
- Implement control return through opposite edge.
- Add emergency stop.
- Prevent self-injected event feedback where possible.

Exit criteria:

- macOS controls Windows.
- Windows controls macOS.
- Move/down/up/wheel work.
- Handoff and return work for all four directions.
- Emergency stop returns control locally.

Suggested duration:

- 7 to 12 engineering days.

## Phase 7: Hardening And QA

Goal:

- Make MVP stable enough for internal use.

Tasks:

- Test fresh macOS permission onboarding.
- Test Windows non-admin flow.
- Test Wi-Fi and wired LAN.
- Add diagnostics panel.
- Add log rotation.
- Tune queue sizes and move coalescing.
- Fix crashes and state-machine dead ends.
- Build installers for macOS and Windows.

Exit criteria:

- 30-minute continuous control test passes.
- Memory stays below `100MB` target on representative systems or documented if WebView overhead exceeds it.
- Startup below `2s`.
- Average switch latency under `20ms` in normal LAN.
- Known limitations documented in README.

Suggested duration:

- 5 to 10 engineering days.

## Recommended Build Order For AI Coding Agent

1. Scaffold Tauri app and module tree.
2. Implement data structures and storage tests.
3. Implement protocol frame parser and tests.
4. Implement discovery service.
5. Implement pairing service.
6. Implement TCP session and heartbeat.
7. Implement platform input spikes.
8. Integrate edge detection and mouse handoff.
9. Build UI flows.
10. Harden permissions and reconnect.

## Manual QA Matrix

Device pairs:

- macOS Apple Silicon -> Windows 11.
- Windows 11 -> macOS Apple Silicon.
- macOS Intel -> Windows 10.
- Windows 10 -> macOS Intel.

Network:

- Same Wi-Fi.
- Wired LAN.
- Wi-Fi with multicast blocked, using IP direct connect.

Layout:

- Mac left, Windows right.
- Windows left, Mac right.
- Mac top, Windows bottom.
- Windows top, Mac bottom.

Permission:

- macOS permissions missing.
- macOS permissions granted.
- Windows normal user.
- Windows with elevated app focused.

## Performance Test Plan

Startup:

- Measure from process launch to UI ready event.
- Target `< 2s`.

Switch latency:

- Measure timestamp at edge detection and timestamp at first successful remote injection.
- Target `< 20ms` on normal LAN.

CPU:

- Idle for 5 minutes.
- Active remote control for 5 minutes.
- Record average and peak.

Memory:

- After startup.
- After 30 minutes idle.
- After 30 minutes active control.

Network:

- Heartbeat RTT.
- Mouse frames per second.
- Dropped/coalesced move frame count.


# Technical Architecture Design

## 1. Recommendation

Use **Tauri + Rust** for V1 MVP.

Rationale:

- System-level input work is the core risk, and Rust has first-class access to native macOS and Windows APIs through FFI and mature crates.
- Tauri keeps the UI layer lightweight by using the system WebView instead of bundling Chromium, making the `< 100MB` memory target more realistic than Electron.
- Rust gives low-latency networking, deterministic event pipelines, and strong type safety for protocol/state-machine code.
- Tauri supports a clean separation between a small HTML/TypeScript UI and a native Rust core.
- The architecture can later extract the Rust core into a daemon/service while keeping the Tauri UI as a shell.

## 2. Framework Comparison

| Option | Strengths | Weaknesses | Fit |
| --- | --- | --- | --- |
| Electron + Node.js | Mature ecosystem, fastest UI iteration, many packages | High memory footprint due to bundled Chromium, native input requires Node native modules or sidecar processes, harder to hit `<100MB` | Not recommended for this MVP |
| Tauri + Rust | Lightweight, strong native API access, low memory, high performance, good packaging story | Rust learning curve, some native APIs require unsafe FFI | Recommended |
| Wails + Go | Simple Go backend, good productivity, native WebView, lower memory than Electron | Go GUI/native input ecosystem is less aligned with low-level macOS event taps and Windows hooks than Rust crates/FFI; fewer desktop plugins | Viable fallback, not primary |

Final choice: **Tauri + Rust + TypeScript frontend**.

## 3. Source Basis

Important platform facts used by this design:

- Apple `CGEvent.tapCreate` creates event taps for observing event streams and may return `NULL` if not permitted. Apple documents that root or assistive access is needed for certain event taps.
- Apple `CGEvent.post` posts Quartz events into an event stream.
- Apple `CGPreflightListenEventAccess` checks whether the app can listen to input events.
- Microsoft `SendInput` synthesizes keyboard, mouse, and button input.
- Microsoft `LowLevelMouseProc` is called when mouse input is about to be posted to a thread input queue.
- Microsoft `SetWindowsHookEx` installs hook procedures such as `WH_MOUSE_LL`.
- Microsoft `GetSystemMetrics` exposes screen and virtual screen metrics such as `SM_CXSCREEN`, `SM_CYSCREEN`, and virtual desktop bounds.

Reference links:

- Apple CGEvent tap creation: https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate(tap:place:options:eventsofinterest:callback:userinfo:)
- Apple CGEvent post: https://developer.apple.com/documentation/coregraphics/cgevent/post(tap:)
- Apple input listen preflight: https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess()
- Microsoft SendInput: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
- Microsoft LowLevelMouseProc: https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc
- Microsoft SetWindowsHookEx: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexa
- Microsoft GetSystemMetrics: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsystemmetrics

## 4. High-Level Architecture

```text
+------------------------------+
| Tauri UI                     |
| TypeScript + HTML/CSS        |
| Device list, pairing, layout |
+--------------+---------------+
               |
               | Tauri commands/events
               |
+--------------v---------------+
| Rust Application Core        |
| State machine, config, logs  |
+---+-----------+----------+---+
    |           |          |
    |           |          |
+---v---+   +---v----+  +--v----------------+
| LAN   |   | Realtime| | Platform Input     |
| Discovery | Network | | macOS / Windows    |
+-------+   +--------+ +--------------------+
```

Runtime roles:

- Every client runs the same binary.
- A device can be controller, controlled, or idle.
- The active control direction is determined by session state.
- V1 supports one paired active peer at a time.

## 5. Process Model

V1 uses one desktop app process:

- Tauri process owns app lifecycle, tray/menu, UI window, local storage, network runtime, and input event workers.
- Rust async runtime handles discovery, TCP transport, heartbeats, and reconnect.
- Platform-specific input hooks run on dedicated OS-compatible threads.

Future extraction point:

- Move input/network core to a background daemon or service.
- Keep Tauri UI as control panel.
- Use local IPC between UI and daemon.

## 6. Core Components

### 6.1 UI Layer

Responsibilities:

- Show local device identity and permissions.
- Show discovered devices.
- Start pairing.
- Confirm pairing.
- Configure layout.
- Show connection and control status.
- Provide emergency disconnect/disable control.

Implementation:

- Tauri frontend with TypeScript.
- Use Tauri commands to call Rust core.
- Subscribe to Rust events for device discovery, session state, and permission status.

### 6.2 Application Core

Responsibilities:

- Own app state machine.
- Validate transitions.
- Coordinate pairing, connection, and input workers.
- Persist config.
- Emit UI events.

Key Rust modules:

- `app_state`.
- `config`.
- `identity`.
- `pairing`.
- `session`.
- `logging`.

### 6.3 Discovery Service

Responsibilities:

- Publish local service.
- Browse same service type.
- Maintain discovered peer cache.
- Provide fallback UDP beacon.

Recommended crates:

- Primary mDNS/Zeroconf: evaluate `mdns-sd` or `zeroconf`.
- UDP fallback: `tokio::net::UdpSocket`.

V1 recommendation:

- Start with `mdns-sd` if it works cleanly across macOS and Windows in CI/manual QA.
- Keep UDP broadcast fallback behind a feature flag or module boundary.

### 6.4 Pairing Service

Responsibilities:

- Generate ephemeral pairing code.
- Exchange pairing requests.
- Require both-side confirmation.
- Persist trusted peer record.
- Reject unknown/unconfirmed control connections.

Security model:

- Generate stable local device key pair on first run.
- Pairing stores peer public key and device ID.
- V1 can use TLS with self-signed local certificates or Noise protocol if implementation cost is acceptable.
- If cryptographic transport is deferred, document it as V1 beta limitation and keep LAN-only warning in UI.

Recommendation:

- Use `rustls` with pinned peer certificates or a Noise-based handshake.
- Avoid unauthenticated plain TCP for control events.

### 6.5 Realtime Transport

Responsibilities:

- Maintain low-latency bidirectional event channel.
- Send mouse event frames.
- Send heartbeat and session state frames.
- Reconnect automatically.

Final V1 transport recommendation:

- Use **TCP with a compact length-prefixed binary protocol**.
- Disable Nagle via `TCP_NODELAY`.
- Keep a persistent connection after pairing.
- Use mDNS only for discovery, not for realtime events.

Why TCP:

- Reliable ordering is important for down/up and wheel sequences.
- Implementation complexity is much lower than QUIC or custom UDP.
- LAN latency with `TCP_NODELAY` is typically low enough for the `<20ms` target.
- Debugging and reconnect behavior are simpler for MVP.

### 6.6 Platform Input Layer

Responsibilities:

- Observe global mouse movement/button/wheel.
- Determine screen bounds and edge hit.
- Inject remote mouse movement/button/wheel.
- Avoid event feedback loops from self-injected events.

Module boundary:

```rust
trait InputPlatform {
    fn permissions(&self) -> PermissionStatus;
    fn request_permissions(&self) -> Result<()>;
    fn screen_topology(&self) -> Result<ScreenTopology>;
    fn start_capture(&self, tx: InputEventSender) -> Result<InputCaptureHandle>;
    fn inject(&self, event: RemoteMouseEvent) -> Result<()>;
}
```

macOS implementation:

- Listen: `CGEventTapCreate` with mouse event mask.
- Inject: `CGEventCreateMouseEvent`, `CGEventSetIntegerValueField`, `CGEventPost`.
- Permissions: Accessibility and Input Monitoring.

Windows implementation:

- Listen: `SetWindowsHookExW(WH_MOUSE_LL, ...)` and `LowLevelMouseProc`.
- Inject: `SendInput` with `INPUT_MOUSE`.
- Screen metrics: `GetSystemMetrics`, optionally `EnumDisplayMonitors`.
- Permissions: standard user for normal apps; elevated target apps may be blocked.

## 7. Network Transport Comparison

| Transport | Latency | Reliability | Complexity | V1 Suitability |
| --- | --- | --- | --- | --- |
| TCP | Low on LAN with `TCP_NODELAY` | Ordered and reliable | Low | Best V1 choice |
| WebSocket | Low enough but extra framing | Ordered and reliable | Medium | Good for browser compatibility, unnecessary here |
| QUIC | Very low, modern, stream/datagram support | Reliable streams and optional datagrams | High | Good V2 candidate |
| UDP + custom protocol | Lowest theoretical latency | Must build ordering, loss recovery, congestion behavior | High | Not recommended for V1 |

Final recommendation:

- V1: TCP + length-prefixed binary frames + `TCP_NODELAY`.
- V2: consider QUIC only if profiling proves TCP is insufficient or multiple streams/datagrams become valuable.

## 8. Event Pipeline

Local active mode:

```text
OS mouse event -> platform hook -> edge detector -> local OS continues normal handling
```

Handoff:

```text
Edge detector -> session transition -> send ControlEnter -> remote places pointer at entry edge
```

Remote-control mode:

```text
OS mouse event -> platform hook -> normalize as delta/button/wheel -> network frame -> peer injects event
```

Return:

```text
Remote edge detector -> ControlReturn -> original device exits remote-control mode
```

V1 practical detail:

- Capturing and suppressing local mouse movement can be platform-sensitive.
- If full suppression is not stable in early MVP, park the local cursor near the edge while remote control is active and ignore local absolute position except for deltas.
- The acceptance target is user-perceived remote control continuity, not perfect invisible local cursor behavior.

## 9. State Machine

```text
Disconnected
  -> Discovered
  -> Pairing
  -> Paired
  -> Connecting
  -> ConnectedIdle
  -> ControllingRemote
  -> ControlledByRemote
  -> Reconnecting
  -> ConnectedIdle
```

Error transitions:

- Any network failure from `Connecting`, `ConnectedIdle`, `ControllingRemote`, or `ControlledByRemote` moves to `Reconnecting`.
- Permission failure from input startup moves to `Error(MissingPermission)`.
- Pairing rejection returns to `Discovered` or `Paired` depending on previous trust state.

## 10. Permission Architecture

### 10.1 macOS

Accessibility:

- Used to synthesize mouse events and control other apps.
- Needed for reliable `CGEventPost`-based input injection.
- User grants in System Settings > Privacy & Security > Accessibility.

Input Monitoring:

- Used to observe global input events.
- Check with `CGPreflightListenEventAccess`.
- Request with `CGRequestListenEventAccess` where available.
- User grants in System Settings > Privacy & Security > Input Monitoring.

Screen Recording:

- Not required for V1 mouse-only behavior.
- Do not request at first launch.
- Reserve for future screen preview, remote screen awareness, or visual setup assistant.

Onboarding:

- On first launch, run permission preflight checks.
- Show missing permissions as checklist items.
- Provide buttons to open System Settings panes.
- Re-check permission when app regains focus and every few seconds while onboarding is visible.
- If permission cannot be detected after user grants it, instruct user to restart the app.

### 10.2 Windows

Standard user:

- Use `WH_MOUSE_LL` for global mouse observation.
- Use `SendInput` for mouse injection.
- No administrator permission required for normal desktop apps.

Limitations:

- Non-elevated app may not inject into elevated/admin applications because of Windows integrity isolation.
- Secure desktop screens such as UAC prompts are out of scope.
- Anti-cheat or protected applications may block or flag injected input.

Onboarding:

- No permission checklist by default.
- Show warning only when injection fails repeatedly or app detects it is trying to control elevated surfaces.

## 11. Performance Design

Startup:

- Lazy-start discovery and input services after UI shell opens.
- Avoid loading heavy frontend libraries.
- Keep startup config reads small and synchronous only for essential identity/config.

Mouse latency:

- Keep event path in Rust.
- Use compact binary frames.
- Use `TCP_NODELAY`.
- Do not route high-frequency mouse events through the WebView.
- Coalesce mouse move deltas if sender backlog grows.
- Send button and wheel events immediately and never drop them.

CPU:

- Avoid busy polling.
- Event-driven hooks and async sockets.
- Heartbeat interval `1000ms`.
- Discovery beacon interval `1000ms` to `2000ms`.

Memory:

- Tauri instead of Electron.
- Bound queues for input frames.
- Ring-buffer logs in memory with file sink.

## 12. Observability

Logging:

- Use `tracing`.
- Log discovery, pairing, transport, permission, and state transitions.
- Avoid logging raw high-frequency mouse events by default.
- Add debug sampling for mouse event latency.

Metrics:

- Last heartbeat RTT.
- Sent input frames per second.
- Dropped/coalesced move frames.
- Reconnect count.
- Permission status.

Diagnostics UI:

- Hidden or secondary diagnostics panel in V1.
- Export logs button may be V1.1 if packaging supports it.


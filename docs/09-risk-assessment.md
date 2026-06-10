# Risk Assessment

## 1. Highest Risks

| Risk | Impact | Likelihood | Mitigation |
| --- | --- | --- | --- |
| macOS permissions block event capture or injection | Core feature fails | High | Build platform spike first; clear onboarding; test signed builds |
| Local cursor suppression is inconsistent | User experience feels rough | Medium | Use cursor parking and delta forwarding for V1; refine later |
| Windows elevated apps reject injected input | Remote control appears broken in admin apps | Medium | Document limitation; detect repeated injection failure; optional elevated mode later |
| LAN discovery fails on some networks | Users cannot connect easily | High | Provide IP direct connect; mDNS + UDP fallback |
| Mouse latency exceeds target over Wi-Fi | UX degrades | Medium | TCP_NODELAY, compact binary protocol, move coalescing, metrics |
| Self-injected event feedback loop | Cursor jumps or repeats | Medium | Tag/filter injected events where possible; ignore events while injecting; sequence tracking |
| DPI/coordinate mismatch | Pointer lands incorrectly | Medium | Normalize coordinates; test Retina and Windows scaling |
| Security too weak for shared LAN | Unauthorized control risk | Medium | Pairing confirmation, peer key pinning, trusted peer validation |

## 2. macOS Platform Risks

### 2.1 Accessibility Permission

Issue:

- Input injection through Quartz events requires assistive access in many practical cases.

Mitigation:

- Preflight permission at startup.
- Show permission checklist.
- Provide button to open System Settings.
- Re-check when app gains focus.
- Test with packaged app because dev and signed app identities differ.

### 2.2 Input Monitoring Permission

Issue:

- Global input listening may fail if Input Monitoring is not granted.

Mitigation:

- Use `CGPreflightListenEventAccess`.
- Call request API when available.
- Display missing permission state without crashing.

### 2.3 Screen Recording Permission

Issue:

- Requesting unnecessary Screen Recording permission can reduce trust.

Mitigation:

- Do not request Screen Recording in V1.
- Mention it only as future requirement if screen preview/topology features are added.

### 2.4 Event Tap Stability

Issue:

- Event taps can be disabled by the system if callbacks are slow.

Mitigation:

- Keep callback extremely small.
- Push event into lock-free or bounded channel.
- Do not do network I/O or UI calls inside callback.
- Re-enable tap if disabled.

## 3. Windows Platform Risks

### 3.1 Integrity Levels And UIPI

Issue:

- A non-admin process may not inject into higher-integrity admin windows.

Mitigation:

- V1 runs as normal user.
- Document limitation.
- Add warning when active elevated window is suspected or repeated `SendInput` issues occur.
- V2 can offer optional elevated helper if truly needed.

### 3.2 Hook Message Loop

Issue:

- `WH_MOUSE_LL` needs an active message loop and must return quickly.

Mitigation:

- Dedicated thread for hook.
- Minimal callback work.
- Forward events to Rust channel.
- Watchdog restart if hook stops.

### 3.3 Antivirus Or Security Software

Issue:

- Apps that globally hook and inject input may be flagged.

Mitigation:

- Code sign builds when moving beyond internal MVP.
- Keep transparent permission UI.
- Avoid suspicious behaviors such as hidden persistence.
- Provide clear app name and publisher metadata.

## 4. Network Risks

### 4.1 Multicast Blocked

Issue:

- mDNS may be blocked on enterprise or guest Wi-Fi.

Mitigation:

- IP direct connect is a required V1 path.
- UDP broadcast fallback.
- UI should say discovery unavailable and offer IP entry.

### 4.2 Firewall Prompts

Issue:

- Windows Firewall may block incoming TCP.

Mitigation:

- Show connection troubleshooting message.
- Installer can add firewall rule in later build.
- For V1, document manual allow step.

### 4.3 Wi-Fi Packet Loss/Jitter

Issue:

- Movement can feel uneven.

Mitigation:

- Coalesce move events.
- Prioritize button/wheel/control messages.
- Add RTT and dropped/coalesced metrics.

## 5. Protocol And Security Risks

### 5.1 Unauthorized LAN Control

Issue:

- Any LAN peer could attempt connection if transport is unauthenticated.

Mitigation:

- Pairing confirmation.
- Store peer public key.
- Reject unknown peer outside pairing mode.
- Reject peer key mismatch.
- Prefer encrypted/authenticated transport in V1 if schedule allows.

### 5.2 Replay Pairing

Issue:

- Old pairing messages could be replayed.

Mitigation:

- Pairing nonce.
- Short expiry.
- Store recently used nonces during pairing window.
- Code derived from nonces and public keys.

### 5.3 Parser Bugs

Issue:

- Malformed frames could crash the app.

Mitigation:

- Strict max frame size.
- Fuzz/property tests.
- No unwrap in parser.

## 6. Product Risks

### 6.1 UX Around Permissions

Issue:

- macOS permission setup can be frustrating.

Mitigation:

- First screen shows exactly what is missing.
- Avoid requesting unnecessary Screen Recording.
- Provide direct settings buttons.
- Show restart guidance only when needed.

### 6.2 Accidental Handoff

Issue:

- Users may hit the edge accidentally.

Mitigation:

- Corner guard.
- Optional short edge dwell threshold if needed after testing.
- Emergency stop in tray/menu.

### 6.3 User Mental Model

Issue:

- Users may confuse physical screen layout with app layout.

Mitigation:

- Visual two-screen layout editor.
- Highlight active edge.
- Use device names inside monitor rectangles.

## 7. Performance Risks

### 7.1 Memory Target

Issue:

- WebView memory varies by platform; `<100MB` may be tight.

Mitigation:

- Tauri over Electron.
- Keep UI lightweight.
- Avoid large frontend dependencies.
- Measure actual memory early.

### 7.2 Event Backlog

Issue:

- High-frequency mouse movement can overload queue/network.

Mitigation:

- Bounded channels.
- Coalesce moves.
- Preserve button/wheel ordering.
- Drop only stale move deltas.

## 8. Build And Distribution Risks

### 8.1 macOS Signing Identity Changes Permissions

Issue:

- macOS privacy grants are tied to app identity/path/signature.

Mitigation:

- Test dev and packaged builds separately.
- Stabilize bundle ID early.
- Avoid changing signing identity repeatedly during QA.

### 8.2 Windows WebView2 Runtime

Issue:

- Tauri depends on WebView2 on Windows.

Mitigation:

- Document runtime requirement.
- Use Tauri installer option to bootstrap WebView2 if needed.

## 9. Go/No-Go Criteria For MVP

Go:

- Platform input spike passes both OSes.
- Pairing prevents unconfirmed access.
- Handoff works in both directions.
- Reconnect is stable.
- Known limitations are visible in UI/docs.

No-Go:

- macOS injection cannot work reliably after granted permissions.
- Windows hook/injection is blocked on normal apps.
- Mouse button up/down ordering is unreliable.
- App can be controlled by unpaired peer.


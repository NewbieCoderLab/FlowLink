# Risk Assessment

## 1. Highest Risks

| Risk | Impact | Likelihood | Mitigation |
| --- | --- | --- | --- |
| macOS permissions block event capture or injection | Core feature fails | Medium | S1.1 has real preflight/request/settings links and focus refresh; still test signed builds |
| Local cursor suppression is inconsistent | User experience feels rough | Medium | Use cursor parking and delta forwarding for V1; refine later |
| Windows elevated apps reject injected input | Remote control appears broken in admin apps | Medium | Show integrity level in UI; document UIPI limitation; optional elevated mode later |
| LAN discovery fails on some networks | Users cannot connect easily | Medium | mDNS + UDP fallback are implemented; keep IP direct connect as required V1 path |
| Mouse latency exceeds target over Wi-Fi | UX degrades | Medium | TCP_NODELAY, compact binary protocol, move coalescing, metrics |
| Self-injected event feedback loop | Cursor jumps or repeats | Low-Medium | macOS uses `kCGEventSourceUserData = FLOW_TAG`; Windows uses `dwExtraInfo`; keep spike checks |
| DPI/coordinate mismatch | Pointer lands incorrectly | Medium | Normalize coordinates; unit-test Windows virtual desktop math; test Retina and Windows scaling |
| Security too weak for shared LAN | Unauthorized control risk | Medium | Pairing confirmation, peer key pinning, trusted peer validation |

## 2. macOS Platform Risks

### 2.1 Accessibility Permission

Issue:

- Input injection through Quartz events requires assistive access in many practical cases.

Mitigation:

- Preflight permission at startup is implemented for macOS.
- Permission checklist is implemented in the Permissions tab.
- Settings buttons open Accessibility / Input Monitoring pages with Privacy fallback.
- Re-check when app gains focus is implemented in the frontend status hook.
- Test with packaged app because dev and signed app identities differ.

### 2.2 Input Monitoring Permission

Issue:

- Global input listening may fail if Input Monitoring is not granted.

Mitigation:

- `CGPreflightListenEventAccess` is implemented.
- `CGRequestListenEventAccess` is called from the request path.
- Missing permission state is displayed without crashing.
- `examples/mac_input_spike.rs` can validate capture from a terminal process.

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

- Callback only checks the self-injection tag and `try_send`s into a bounded channel.
- Do not do network I/O or UI calls inside callback.
- Re-enable tap if disabled.

## 3. Windows Platform Risks

### 3.1 Integrity Levels And UIPI

Issue:

- A non-admin process may not inject into higher-integrity admin windows.

Mitigation:

- V1 runs as normal user.
- Permissions tab exposes the current process integrity level.
- `bench/results/s1.2-windows.md` documents the normal-user limitation.
- Add warning when active elevated window is suspected or repeated `SendInput` issues occur.
- V2 can offer optional elevated helper if truly needed.

### 3.2 Hook Message Loop

Issue:

- `WH_MOUSE_LL` needs an active message loop and must return quickly.

Mitigation:

- Dedicated thread for hook.
- Minimal callback work.
- Forward events to Rust channel.
- Capture handle posts `WM_QUIT`, unhooks with `UnhookWindowsHookEx`, and clears the sender on shutdown.
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
- UDP broadcast fallback is implemented and covered by local tests.
- Run two-machine discovery smoke on representative routers before MVP go/no-go.
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

# Product Requirements Document

## 1. Product Summary

Product name placeholder: `Mac22Win`.

`Mac22Win` is a cross-platform desktop app for macOS and Windows. After installing the client on two computers in the same local network, users can move the mouse from one computer to another as if both computers were part of one multi-monitor workspace.

V1 MVP focuses on reliable LAN discovery, pairing, layout configuration, and low-latency mouse control. It does not include accounts, public network traversal, commercial licensing, file transfer, clipboard sync, or keyboard sharing.

## 2. Target Users

- Users who work with one Mac and one Windows PC at the same desk.
- Developers, designers, analysts, support staff, and power users who frequently switch between two computers.
- Users who want mouse continuity without buying hardware KVM devices.

## 3. Goals

- Allow two devices in the same LAN to discover each other automatically.
- Allow users to pair devices safely through confirmation and pairing code.
- Allow users to define relative screen layout: left, right, top, or bottom.
- Allow the local mouse to cross the configured screen edge and control the remote machine.
- Keep perceived switch latency below `20ms` in a normal wired or strong Wi-Fi LAN.
- Keep memory below `100MB` in steady state.
- Start the app in less than `2s` on supported machines.

## 4. Non-Goals

- No account system.
- No cloud service.
- No public internet traversal.
- No relay server.
- No file transfer.
- No clipboard sync in V1.
- No keyboard sharing in V1.
- No multi-device mesh beyond two devices in V1.
- No mobile platform support.

## 5. Supported Platforms

- macOS on Intel Mac.
- macOS on Apple Silicon.
- Windows 10.
- Windows 11.

Minimum recommended OS versions:

- macOS 12 Monterey or later for V1 QA baseline.
- Windows 10 22H2 or later for V1 QA baseline.

Older systems may work if the system APIs are available, but they are not part of V1 acceptance testing.

## 6. Core User Stories

### 6.1 First Launch

As a user, I can open the app and see the local device name, OS type, network status, and permission status.

Acceptance criteria:

- App launches in less than `2s`.
- The app displays whether mouse monitoring and mouse control permissions are ready.
- If required permission is missing, the app shows a clear action to open the relevant system settings page.

### 6.2 Discover Devices

As a user, I can see compatible devices in the same LAN without manually typing an IP address.

Acceptance criteria:

- The device list updates within `3s` after another client starts on the same LAN.
- Each discovered device shows device name, OS type, IP address, app version, and last seen time.
- Stale devices disappear or become inactive after `10s` without discovery beacons.

### 6.3 Pair Devices

As a user, I can pair with another device through a pairing code or direct IP.

Acceptance criteria:

- Pairing must require confirmation on both sides.
- Pairing code expires after `120s`.
- Failed or expired pairing attempts do not create a trusted device record.
- Successful pairing persists device identity and display name locally.

### 6.4 Configure Layout

As a user, I can set whether the remote device is left, right, above, or below the local device.

Acceptance criteria:

- User can select one of `left`, `right`, `top`, `bottom`.
- Layout persists after restart.
- The UI displays a visual two-screen layout preview.
- The selected edge determines when mouse handoff is triggered.

### 6.5 Mouse Handoff

As a user, I can move the pointer to the configured edge and start controlling the remote device.

Acceptance criteria:

- Local edge detection triggers only when the active paired device is connected.
- After handoff, mouse movement, down, up, and wheel events are sent to the remote device.
- Remote device injects system mouse input.
- Moving back through the opposite edge returns control to the original machine.
- The UI shows which device is currently controlled.

### 6.6 Connection Recovery

As a user, I do not need to manually reconnect after a short network interruption.

Acceptance criteria:

- Heartbeat detects a dead connection within `3s`.
- App attempts automatic reconnect with exponential backoff.
- On reconnect, app restores paired session and layout.
- If reconnect fails for more than `60s`, UI shows a disconnected state with a manual reconnect action.

## 7. Functional Requirements

### 7.1 Device Discovery

Required:

- LAN automatic discovery.
- Show device name.
- Show OS type.
- Show IP address.
- One-click connect from discovery list.

Recommended V1 discovery mechanism:

- Use `mDNS`/Zeroconf as the primary discovery mechanism.
- Add UDP broadcast as a fallback where multicast is blocked.

Discovery service name:

```text
_mac22win._tcp.local
```

Discovery TXT fields:

```text
device_id=<stable-device-id>
device_name=<user-visible-name>
os=macos|windows
arch=x86_64|aarch64
app_version=0.1.0
protocol_version=1
pairing=t|f
```

### 7.2 Device Pairing

Required pairing methods:

- Pairing code.
- IP direct connect.

Required safety controls:

- Both devices display a pairing confirmation dialog.
- Pairing code must be short-lived.
- Pairing request must show remote device name, OS, IP address, and pairing code fingerprint.
- Trusted peer identity must be persisted.

V1 security target:

- Prevent accidental connection.
- Prevent casual LAN misuse.
- Do not claim enterprise-grade security until certificate pinning and stronger authentication are fully implemented.

### 7.3 Screen Layout

Required layout options:

- Remote on left.
- Remote on right.
- Remote on top.
- Remote on bottom.

UI requirements:

- Two monitor cards showing local and remote device names.
- Direction selector using a segmented control.
- Save/apply button.
- Test area indicator showing the active edge.
- Status badge: `Ready`, `Missing Permission`, `Disconnected`, `Controlling Remote`, `Controlled By Remote`.

### 7.4 Mouse Control

Main controller must listen for:

- Mouse movement.
- Mouse down.
- Mouse up.
- Mouse wheel.

Main controller must detect:

- Left edge: `x <= virtual_screen_min_x`.
- Right edge: `x >= virtual_screen_max_x - 1`.
- Top edge: `y <= virtual_screen_min_y`.
- Bottom edge: `y >= virtual_screen_max_y - 1`.

Controlled device must receive and inject:

- `MouseMove`.
- `MouseDown`.
- `MouseUp`.
- `Wheel`.

V1 behavior:

- While local device is active, keep normal mouse behavior.
- On edge trigger, enter remote-control mode.
- In remote-control mode, forward deltas and button/wheel events to remote.
- Local cursor should be parked just inside the transfer edge or hidden only if platform support is reliable.
- Remote pointer position starts at the corresponding opposite edge.

### 7.5 Reconnect And State Sync

Required:

- Heartbeat.
- Connection status.
- Current control owner.
- Current pointer position.
- Reconnect after network interruption.

State model:

- `Disconnected`.
- `Discovered`.
- `Pairing`.
- `Paired`.
- `Connecting`.
- `ConnectedIdle`.
- `ControllingRemote`.
- `ControlledByRemote`.
- `Reconnecting`.
- `Error`.

## 8. Non-Functional Requirements

Performance targets:

- App startup: `< 2s`.
- Mouse switch latency: `< 20ms` under normal LAN conditions.
- Steady CPU: ideally `< 2%` idle, `< 8%` during active remote control on typical hardware.
- Steady memory: `< 100MB`.
- Discovery update: peer visible within `3s`.
- Heartbeat failure detection: within `3s`.

Reliability targets:

- No crash when permissions are missing.
- No crash when peer disappears.
- No unbounded memory growth during continuous mouse movement.
- Protocol remains compatible within the same major protocol version.

Usability targets:

- First successful pairing should take less than `2 minutes`.
- Permission setup should be visible and recoverable.
- User can disable remote control quickly from the tray/menu.

## 9. Permission Requirements

macOS:

- Accessibility: required to inject mouse input and usually required for global event control.
- Input Monitoring: required to observe global mouse/keyboard input in newer macOS privacy model.
- Screen Recording: not required for V1 mouse-only MVP; reserve for future screen topology detection, screen preview, or remote display features.

Windows:

- Normal user privileges should be enough for `WH_MOUSE_LL` listening and `SendInput` injection into same-integrity or lower-integrity apps.
- Administrator is not required for V1 normal operation.
- Events may not control elevated/admin apps from a non-elevated process because of Windows integrity levels and UIPI restrictions.

## 10. Success Metrics

MVP success:

- Two devices can discover each other in a normal home/office LAN.
- Pairing succeeds with pairing code and IP direct connect.
- User can configure layout and persist it.
- Mouse handoff works both directions.
- Remote pointer movement feels immediate in normal LAN conditions.
- App remains under memory target during a 30-minute test.

## 11. Release Criteria

V1 can be considered ready when:

- macOS-to-Windows and Windows-to-macOS mouse control both pass manual QA.
- Pairing and reconnect pass manual QA over Wi-Fi and wired LAN.
- Permission onboarding is tested on fresh macOS install/user profile.
- Windows non-admin install and run path works.
- Logs provide enough information to diagnose discovery, pairing, and control failures.


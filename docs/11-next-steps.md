# 后续技术路线 v0.2

文档版本：`v0.2`

撰写时间：2026/06/11

适用范围：在 [00-master-plan-zh-CN](./00-master-plan-zh-CN.md) 与 [08-mvp-roadmap](./08-mvp-roadmap.md) 已落地 V1 设计的前提下，把"骨架就绪 → 端到端可用"这段路细化为可逐项打勾的 Sprint 级实施清单。

本篇不替换 08，只在 08 的 Phase 0~7 之上增加：
- 当前仓库已完成 / 占位 / 缺失的精确清单。
- 关键技术决策的补充（V1 加密通道选型为 Noise XX + Ed25519 公钥 pin）。
- 切到具体 Sprint 的实施任务、改动文件、新增依赖、验收脚本和测试用例。
- 跨 Sprint 的支撑工作（CI、性能基线、QA Matrix 自动化）。

阅读顺序：先读 §2 现状，再读 §3 决策更新，然后按 §4 顺序推进 Sprint，§5/§6 在每个 Sprint 完成后回顾。

---

## 1. 与既有文档的关系

| 既有文档 | 关系 |
| --- | --- |
| `00-master-plan-zh-CN.md` | 本篇是其落地视角，所有协议、状态机、目录结构以 00 为准 |
| `01-prd.md` ~ `07-project-structure.md` | 数据结构、协议、存储、目录沿用，不重复 |
| `08-mvp-roadmap.md` | Phase 0~7 不变，本篇把每个 Phase 拆成 1~2 个 Sprint |
| `09-risk-assessment.md` | 沿用风险矩阵，本篇 §6 补当前缓解状态 |
| `10-v2-v3-evolution.md` | 不变，本篇 §7 给 V1 → V1.1 衔接的取舍建议 |

如本篇与 00 冲突，以 00 为准；如 00 与 PRD 冲突，以 PRD 为准。

---

## 2. 当前完成情况

口径：以 `src-tauri/src` 与 `src` 实际代码为准，不以模块文件存在与否判断完成度。

### 2.1 已落地

工程层：

- `Tauri 2 + React + TypeScript` 工程能启动，命令注册齐全，前端通过 `useAppStatus` 读取 `get_app_status`。
- Rust 模块树按 `07-project-structure.md` 切分到位，`lib.rs` 装配 `AppContext` 并注入 Tauri state。
- 本地存储：`StorageManager` 已实现 `identity / config / trusted_peers / layouts` 的读写，写入走 `*.tmp -> rename` 原子路径。
- 本地存储恢复：坏 JSON 会备份为同目录 `.corrupt.<timestamp>` 并写入默认值；IO 读取失败仍返回 `StorageError::Io`。
- 数据结构：`AppConfig / NetworkConfig / DiscoveryConfig / LayoutConfig / TrustedPeer / DeviceIdentity / SessionSnapshot / PermissionStatus` 已定义。
- 测试基线：`src-tauri/tests/` 已覆盖 framing、pairing code、storage、AppContext 启动发现列表；当前 `cargo test` 共 18 个集成测试全绿。
- 质量基线：本地 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`npm run build` 均已通过。
- CI 基线：`.github/workflows/ci.yml` 已配置 macOS + Windows 双 runner，执行 Rust fmt/check/clippy/test 与前端 build。
- 日志基线：`telemetry::logging::init_logging` 已接入 `tracing-appender` daily rolling appender，优先写入 Tauri `app_log_dir()`，并用 `Once` 防重复初始化。
- 启动清理：真实 `AppContext::load_or_default` 不再注入 demo peer；demo 设备仅保留在前端 mock fallback 中。

协议与算法层：

- 帧编解码：`encode_frame / decode_frame` 已实现，header 14 字节（length u32 BE + version u16 + type u16 + flags u16 + seq u32），最大帧 64 KiB，截断帧返回 `Incomplete`。
- `MessageType` 枚举已经按 00 §5.4 列齐 17 个类型。
- 配对码：`pairing::code::generate_pairing_code` 按 00 §5.6 公式实现，输出 6 位十进制。
- 边缘检测：`input::edge::is_handoff_edge_hit` 已支持 `corner_guard_px` 与 `edge_thickness_px`。
- 重连退避：`network::reconnect::next_backoff_ms` 实现指数退避（未含 jitter）。
- 发现缓存：`DiscoveryCache` 实现 upsert、stale 过滤与 `evict_stale`，可驱动 `device:discovered` / `device:stale` 事件。
- 平台输入抽象：`InputPlatform` trait、`NoopInputPlatform`、`AppContext` 平台输入持有与 `get_screen_topology` Tauri 命令已接通；`PermissionStatus` 由具体 `InputPlatform` 返回，不再从 identity 推导。
- macOS 输入雏形：`MacInputPlatform` 已有 Event Tap 监听、CGEvent 注入、自注入标记过滤、屏幕拓扑查询和权限查询/请求入口，仍待 spike 实机验收。
- Windows 输入雏形：`WinInputPlatform` 已有 `WH_MOUSE_LL` 监听、`SendInput` 注入、自注入标记过滤、DPI awareness 与显示器枚举，仍待 Windows 实机编译/验收。

UI 层：

- 6 Tab 偏好面板：Overview / Devices / Layout / Permissions / Network / About。
- 中英文 i18n（`src/app/i18n.ts`）。
- Tauri 调用失败时回退到 `mockStatus`，可在浏览器中预览（`npm run web:dev`）。

### 2.2 占位、缺失或仅有 stub

热路径相关（影响 V1 是否能用）：

| 位置 | 现状 | 说明 |
| --- | --- | --- |
| `discovery/mdns.rs` | 已实现 `_mac22win._tcp.local.` 发布、浏览解析和本机过滤 | 仍需双机 LAN smoke |
| `discovery/udp.rs` | 已实现 JSON announce 发送、监听解析和 UDP fallback | 仍需双机 LAN smoke |
| `network/listener.rs` | 仅 `format!` 出地址 | 没有 `TcpListener::bind` 与 accept loop |
| `network/connector.rs` | 仅 `format!` 出地址 | 没有连接尝试 |
| `network/heartbeat.rs` | 仅返回 `Duration` 常量 | 没有心跳 ping/pong |
| `input/macos.rs` | Event Tap / CGEvent 注入初版 | 仍缺 spike 实机验收、关闭机制、方向/按钮细节确认 |
| `input/windows.rs` | Hook / SendInput 初版 | 仍缺 Windows 实机编译、关闭机制、侧键注入和 DPI 验收 |
| `platform/macos_permissions.rs` | 已接 `AXIsProcessTrustedWithOptions` / `CGPreflightListenEventAccess` | 设置面板 URL 和 UI 刷新闭环仍需 S1 验证 |
| `platform/windows_permissions.rs` | 普通输入返回 granted，并读取当前进程 integrity level | 仍需 S6/S7 验证管理员窗口 UIPI 限制的用户提示 |
| `session/controller.rs` | 仅 `emergency_disconnect` | 没有完整状态机驱动 |
| `pairing/flow.rs` | `PairingFlow::new` 生成 id 与过期时间 | 没有 request/response/confirm 三段，没有 trust 写入 |
| `identity/keys.rs` | `generate_public_key_stub` 8 行 | 没有 Ed25519 真密钥对 |
| `storage/secret.rs` | 5 行 stub | 没有 Keychain / DPAPI 后端 |
| `telemetry/metrics.rs` | 仅诊断快照字段 | 没有 RTT / fps / drop / CPU / RSS 采集 |
| `ui_api/commands.rs` 中 `start_pairing / confirm_pairing / connect_peer` | 仍直接 `Ok(())` 或仅创建 PairingFlow | 不连接 Rust Core |

测试与发布相关：

- 没有 `tauri build` 配置签名、公证、打包脚本。
- `tracing` file appender 已有 daily rolling；7 天保留策略尚未实现。
- 没有 `start_minimized`、托盘、紧急断开快捷键。

### 2.3 偏离设计文档的地方

需要在后续 Sprint 中修正：

- `protocol::frame::HEADER_LEN = 10`，按 00 §5.3 是 10 字节（`version + type + flags + seq` = 2+2+2+4），与 BUF 写入一致；但若把 `length u32` 也算进 header 的话总长是 14。代码以"length 不计入 header"为口径，文档需明确这一点（已在 §5.1 锁定）。
- `DiscoveredPeer::demo_peer()` 函数仍保留给 mock/未来测试，但真实 `AppContext::load_or_default` 已不再硬编码注入。
- `PermissionStatus` 已改为完全由 `InputPlatform::permissions()` 提供；macOS、Windows、noop 分别返回对应平台状态。
- `MessageType::Other(u8)` 在 `MouseButton` 中带元组数据但 `#[repr(u16)]` 仅在 `MessageType` 上，注意序列化路径 — 鼠标按钮 wire format 在 §5.2 中重新约定。

---

## 3. 关键技术决策更新

### 3.1 通道加密：Noise XX + Ed25519 公钥 pin

V1 在 TCP 之上跑 Noise Protocol Framework 的 `XX` 模式。

理由：

- LAN 不等于可信。任何能加入同网段的设备都能尝试连。仅靠 `Hello` 中的 `device_id` 字符串无法防中间人。
- 配对阶段已经为每台设备生成了长期 Ed25519 公钥，自然适合做 pinned identity。
- Noise XX 在 1.5 RTT 内完成相互认证 + 派生 ChaCha20-Poly1305 流密钥，开销极小（每帧 16 字节认证标签），LAN 环境下不影响 `< 20ms` 切换目标。
- Synergy / Barrier 是无加密 TCP，被多次诟病；Mouse Without Borders 自研轻量加密；本项目按现代标准走 Noise，未来公网穿透也能复用。

不引入 TLS 的理由：

- 不需要 PKI，不需要证书签发流程。
- TLS 握手代码量大于 Noise，依赖 `rustls` 等较重的库。
- 公钥 pin 比证书校验更贴近 V1 的"配对即信任"语义。

替代库：`snow` crate（纯 Rust，活跃维护，支持 Noise XX、IK、NK 等）。

加密包络与现有帧的关系：

```text
Application Frame  (header + payload)
       │
       ▼
Noise Cipher State (encrypt with associated data = none)
       │
       ▼
Network Frame      length u32 BE | encrypted bytes (header + payload + 16 byte tag)
```

口径：长度前缀本身不加密；length 字段之后的全部字节都是 Noise 密文。`MAX_FRAME_LENGTH = 64 KiB` 表示密文长度上限。

握手前的 `Hello` 仍然走明文，用于：

- 协议版本协商（不兼容直接断）。
- 取出对端 `device_id`，查 `trusted_peers.json` 找到对应 pinned key。
- 从 `Hello` 中读到对端 `static_public_key`，与 pinned key 比对，不匹配立刻关连接。

明文 Hello 的字段集合保持 00 §5.5 不变，新增 `noise_pattern: "XX"` 与 `noise_prologue_hash: [u8; 32]`。Prologue 用 `SHA256(min(device_id_a, device_id_b) || max(device_id_a, device_id_b) || protocol_version_be)`，把握手绑定到这次会话。

### 3.2 长连接与控制通道复用

V1 单 TCP 连接承载所有消息，按 `MessageType` 复用：

- 配对相关：`PairingRequest / Response / Confirm / Reject`。
- 会话握手：`Hello / HelloAck / SessionStart / SessionState`。
- 控制权移交：`ControlEnter / ControlLeave`。
- 输入：`MouseMove / MouseButton / MouseWheel`。
- 心跳：`HeartbeatPing / Pong`。
- 错误与关闭：`Error / Goodbye`。

不在 V1 引入多 stream 或 datagram；`MouseMove` 的合并完全在发送端 `bounded mpsc` 里做。

### 3.3 内部消息总线

Rust Core 内部统一用 `tokio::sync::mpsc` + `broadcast`：

```text
PlatformInput ──tx──▶ session::Controller ──tx──▶ NetworkWriter
                                                     │
                                                     ▼
                                                  TcpStream
TcpStream ──▶ NetworkReader ──tx──▶ session::Controller ──tx──▶ PlatformInject
                                              │
                                              ▼
                                         tauri::AppHandle::emit
```

要点：

- `bounded(256)` 的 `mpsc` 给输入事件，溢出时丢弃旧的 `MouseMove`，但 `MouseButton/MouseWheel` 永远入队。
- `broadcast(64)` 给前端事件，用于多 Tab 订阅。
- 所有 channel 关闭都视作"对端走了"信号，按状态机处理。

### 3.4 自注入事件过滤

策略组合：

- macOS：注入时给 `CGEvent` 设置自定义 `kCGEventSourceUserData` 标记位（约定值 `0x464C4F57` "FLOW"），监听回调里见此标记即跳过。
- Windows：通过 `INPUT.mi.dwExtraInfo = SIGNATURE`（约定值 `0xF10F1117`），低级 Hook 中读取 `MSLLHOOKSTRUCT.dwExtraInfo` 并跳过。

补充：远控期间，被控端的本地光标移动事件全部丢弃不上发，避免反向回环。

### 3.5 坐标与 DPI

V1 内部一律用浮点逻辑像素：

- macOS：CoreGraphics 已是逻辑坐标，直接使用。
- Windows：进程使用 `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)`，按显示器逻辑像素计算；注入绝对坐标时再换算到 `0..65535` 的虚拟桌面归一化值。

跨设备口径：发送 `MouseMove` 用 delta（不传绝对坐标），仅在 `ControlEnter` 时传一次"远端入口边的相对位置"。这样无需在两端做绝对坐标对齐。

---

## 4. Sprint 级实施清单

每个 Sprint 给出：目标、范围、任务、改动文件、新增依赖、验证方法、退出标准、估时。

总时序如下：

```text
S0 Test Harness ─┐
                 ├─ S1 Platform Input Spike ─┐
                 │                           ├─ S5 Layout & Edge ─┐
                 ├─ S2 Discovery ────────────┤                    ├─ S6 Handoff ─ S7 Hardening ─ S8 Packaging
                 ├─ S3 Pairing ──────────────┤                    │
                 └─ S4 TCP + Noise ──────────┘                    │
```

S2/S3/S4 可在 S1 完成 Spike 后并行，S6 必须等 S1/S4/S5 全绿。

### Sprint S0：测试与清理基线（已完成，2026-06-11）

目标：让所有后续 Sprint 都能跑测试、跑 lint、做端到端 dry-run。

任务：

- [x] 在 `src-tauri/Cargo.toml` `[dev-dependencies]` 加 `tokio-test`、`pretty_assertions`、`assert_matches`。
- [x] 在 `src-tauri/tests/` 新增 `framing_tests.rs`、`pairing_code_tests.rs`、`storage_tests.rs`，并补充 `app_context_tests.rs`。
- [x] 为现有已实现模块补单测：
  - `framing`：encode→decode 圆环；超大 payload 拒绝；截断帧返回 `Incomplete` 而非 panic；坏长度字段拒绝。
  - `pairing::code::generate_pairing_code`：交换 a/b 顺序结果一致；nonce 不同结果不同；输出永远 6 位。
  - `storage::files`：tmp→rename 原子；坏 JSON 备份为 `.corrupt.<ts>` 后用默认值启动；IO 读取失败保持 `StorageError::Io`。
- [x] 移除 `discovery::mod::DiscoveredPeer::demo_peer` 的硬编码注入（`AppContext::load_or_default` 中那一行 `discovery.upsert(DiscoveredPeer::demo_peer())`）。在前端 mock fallback 里依然保留 demo peer，以便 `npm run web:dev` 体验不变。
- [x] 新增 `.github/workflows/ci.yml`：macOS + Windows 双 runner，跑 `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test`、`npm run build`。
- [x] `telemetry::logging::init_logging` 接入 `tracing-appender` + `RollingFileAppender`，按天滚动到 `<app_log_dir>/app.log`。

备注：`tracing-appender` 已完成 daily rolling；“保留 7 天”的自动清理策略未在 S0 实现，留给 S7/S8 hardening。

改动文件：

```text
src-tauri/Cargo.toml
src-tauri/tests/framing_tests.rs        (new)
src-tauri/tests/pairing_code_tests.rs   (new)
src-tauri/tests/storage_tests.rs        (new)
src-tauri/src/storage/files.rs          (read_or_default 加 corrupt backup)
src-tauri/src/app/context.rs            (移除 demo_peer)
src-tauri/src/telemetry/logging.rs      (RollingFileAppender)
.github/workflows/ci.yml                (new)
```

新增依赖：`tracing-appender = "0.2"`、`tokio-test = "0.4"`、`pretty_assertions = "1"`、`assert_matches = "1"`。

验证：

```bash
cd src-tauri && cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
npm run build  # 仓库根目录
```

退出标准：

- `cargo test` 至少 12 个用例全绿；当前 18 个集成测试 + 2 个 logging 单测全绿。
- 本地 `cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`、`npm run build` 全绿。
- 双平台 CI 配置已就绪；远端首次出绿需等待 push / PR 后确认。
- 启动时不再插入虚假设备，并由 `app_context_tests.rs` 防回归。

### Sprint S1：平台输入 Spike（5 天）

目标：完成 00 §8 Phase 0 的真实代码，证明 macOS 与 Windows 都能监听 + 注入鼠标事件并测得本机 capture→inject 延迟。

#### S1.1 macOS 监听与注入（实现已完成，手动 QA 待勾选）

任务：

- [x] `src-tauri/src/input/macos.rs` 实现 `MacInputPlatform`：
  - 创建独立 std::thread 运行 `CFRunLoop`。
  - 用 `CGEventTapCreate(kCGSessionEventTap, kCGHeadInsertEventTap, kCGEventTapOptionListenOnly, mask, callback, refcon)` 监听 `mouseMoved | leftMouseDown | leftMouseUp | rightMouseDown | rightMouseUp | otherMouseDown | otherMouseUp | scrollWheel`。
  - `callback` 内只做坐标读取 + 自注入标记检查 + `mpsc::Sender::try_send(LocalMouseEvent)`，绝不阻塞、不打 log、不分配大对象。
  - 注入路径：`CGEventCreateMouseEvent` / `CGEventSetIntegerValueField(kCGEventSourceUserData, FLOW_TAG)` / `CGEventPost(kCGHIDEventTap)`。
  - 屏幕拓扑：`CGGetActiveDisplayList` + `CGDisplayBounds`，组装 `ScreenTopology`。
- [x] `src-tauri/src/platform/macos_permissions.rs`：
  - `accessibility_status()` 调 `AXIsProcessTrustedWithOptions(kAXTrustedCheckOptionPrompt: false)`。
  - `input_monitoring_status()` 调 `CGPreflightListenEventAccess()`（10.15+）。
  - `request_input_monitoring()` 调 `CGRequestListenEventAccess()`。
  - `open_settings_pane(kind)` 用 `open x-apple.systempreferences:com.apple.preference.security?...` 打开 Accessibility / Input Monitoring，并 fallback 到 Privacy 总页。
- [x] `src-tauri/src/input/types.rs` 中的 `LocalMouseEvent` 直接复用；`CaptureHandle` 增加最小 shutdown hook，macOS capture drop 时会请求 `CFRunLoop` 停止并短暂 join。
- [x] `examples/mac_input_spike.rs` 支持：
  - 默认监听并统计 1000 个 move/down/up/wheel 事件。
  - `--inject-move` 注入 100 次小幅移动并打印 median/p95 调用耗时。
  - `--inject-click` 在当前鼠标位置注入一次 left click。
  - `--verify-self-filter` 注入带 FLOW_TAG 的事件并确认 Event Tap 不回收。
  - `--topology` 打印显示器 bounds / scale / primary。

改动文件：

```text
src-tauri/src/input/macos.rs           (实质化)
src-tauri/src/input/mod.rs             (定义 trait InputPlatform、PlatformInputManager)
src-tauri/src/platform/macos_permissions.rs
src-tauri/Cargo.toml
src-tauri/Info.plist                   (new, NSAppleEventsUsageDescription 之类的不需要，但 dev build 提示文案可加)
```

新增依赖：

```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.24"
core-foundation = "0.10"
objc2 = "0.5"
objc2-app-kit = "0.2"
```

验证：

- `cargo run --bin flowlink` 后授予 Accessibility + Input Monitoring，前端 Permissions Tab 应在 1 秒内显示 `granted`。
- 在 mac 上手动跑 `examples/mac_input_spike.rs`：监听并打印鼠标事件 1000 次，再用 `--inject-move` / `--inject-click` / `--verify-self-filter` / `--topology` 验证注入、自过滤与屏幕拓扑。
- QA 记录见 `bench/results/s1.1-macos.md`。

退出标准：

- 监听到 move/down/up/wheel 四类事件（需手动 QA 勾选）。
- `CGEventPost` 能让目标 app（如 TextEdit）真实响应点击（需手动 QA 勾选）。
- 自注入事件被 `kCGEventSourceUserData == FLOW_TAG` 过滤，回调中不再循环（`--verify-self-filter`）。
- spike 中位调用延迟可由 `--inject-move` 输出；是否满足 < 1ms 需实机记录。

#### S1.2 Windows 监听与注入（2 天）

任务：

- `src-tauri/src/input/windows.rs`：
  - 独立线程跑 `SetWindowsHookExW(WH_MOUSE_LL, hook_proc, hinst, 0)` + `GetMessageW` 消息循环。
  - `hook_proc` 内用 `MSLLHOOKSTRUCT.dwExtraInfo == FLOW_SIGNATURE` 跳过自注入；其它事件 `try_send` 到 channel；调用 `CallNextHookEx` 链式传递。
  - 注入：构造 `INPUT { type: INPUT_MOUSE, mi: MOUSEINPUT { ..., dwExtraInfo: FLOW_SIGNATURE } }`，调 `SendInput`。
  - 移动事件优先用 `MOUSEEVENTF_MOVE` 配 delta；`ControlEnter` 时一次性用 `MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK` 锁定起点。
  - 滚轮：`MOUSEEVENTF_WHEEL` 用 `mouseData = WHEEL_DELTA * notches`；横向滚轮用 `MOUSEEVENTF_HWHEEL`。
  - 屏幕拓扑：`EnumDisplayMonitors` + `GetMonitorInfoW` + `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)`。
- `src-tauri/src/platform/windows_permissions.rs`：检测当前进程的完整性级别 `GetTokenInformation(TokenIntegrityLevel)`，给前端提示"普通用户即可"或"以管理员运行"。

改动文件：

```text
src-tauri/src/input/windows.rs
src-tauri/src/platform/windows_permissions.rs
src-tauri/Cargo.toml
```

新增依赖：

```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_UI_Input_KeyboardAndMouse",
  "Win32_UI_WindowsAndMessaging",
  "Win32_UI_HiDpi",
  "Win32_Graphics_Gdi",
  "Win32_System_Threading",
  "Win32_Security",
] }
```

验证：

- `examples/win_input_spike.rs` 默认启动后能打印 1000 个鼠标 hook 事件。
- `--inject-move` 回放 100 个可见移动事件到桌面。
- `--inject-click` 依次注入 left / right / middle click。
- `--inject-wheel` 验证 vertical wheel up/down 与 horizontal wheel right/left。
- `--verify-self-filter` 验证 `dwExtraInfo == FLOW_SIGNATURE` 的自注入事件不会进入 capture 队列。
- `--topology` / `--warp-corners` 用于多显示器 bounds 与绝对定位人工检查。
- QA 记录见 `bench/results/s1.2-windows.md`。

退出标准：

- 在 Win10 22H2 与 Win11 上都能监听 + 注入（需手动 QA 勾选）。
- 高 DPI 多显示器配置下绝对定位误差 ≤ 1 像素（`--warp-corners` 人工验证）。
- Permissions Tab 显示 Windows 当前 integrity level，用于解释管理员窗口 UIPI 限制。

#### S1.3 平台抽象与降级（已完成，手动跨机 QA 待勾选）

任务：

- [x] `src-tauri/src/input/mod.rs` 定义 trait：

  ```rust
  pub trait InputPlatform: Send + Sync {
      fn permissions(&self) -> PermissionStatus;
      fn request_permissions(&self, kind: PermissionKind) -> Result<()>;
      fn screen_topology(&self) -> Result<ScreenTopology>;
      fn start_capture(&self, tx: mpsc::Sender<LocalMouseEvent>) -> Result<CaptureHandle>;
      fn inject(&self, event: RemoteMouseEvent) -> Result<()>;
      fn warp_cursor(&self, position: Point) -> Result<()>;
  }

  pub fn platform_input() -> Box<dyn InputPlatform> {
      #[cfg(target_os = "macos")] return Box::new(macos::MacInputPlatform::new());
      #[cfg(target_os = "windows")] return Box::new(windows::WinInputPlatform::new());
      #[cfg(not(any(target_os = "macos", target_os = "windows")))]
      return Box::new(noop::NoopInputPlatform);
  }
  ```

- [x] `noop.rs` 给 Linux/CI 测试环境用，权限状态返回 `Unsupported`，监听/注入/warp 返回 `InputError::Unsupported`，屏幕拓扑返回空拓扑且不 panic。
- [x] `PermissionStatus` 完全来自 `InputPlatform::permissions()`；Windows 状态由 `platform/windows_permissions.rs::permission_status()` 组装，包含 `windows_integrity_level`。
- [x] 新增 `get_screen_topology` Tauri 命令，后端返回 `UiScreenTopology` JSON。
- [x] 前端新增 `UiRect / UiDisplayInfo / UiScreenTopology` 类型和 `getScreenTopology()` 调用。
- [x] `InputError` 统一映射为 recoverable `UiError` code：`input_unsupported`、`input_permission_denied`、`input_capture_already_running`、`input_platform_error`。
- [x] 新增跨平台 smoke 入口 `src-tauri/examples/platform_input_smoke.rs`，同一命令可在 macOS/Windows/noop 上检查权限、拓扑、监听与注入。
- [x] CI 明确在 macOS + Windows runner 上执行 `cargo check --all-targets`、fmt、clippy、test。

新增/更新测试：

```text
src-tauri/tests/noop_input_tests.rs
src-tauri/tests/ui_screen_topology_tests.rs
src-tauri/tests/ui_input_error_tests.rs
```

本地验证：

```powershell
C:\Users\25994\.cargo\bin\cargo.exe test --test noop_input_tests
C:\Users\25994\.cargo\bin\cargo.exe test --test ui_screen_topology_tests
C:\Users\25994\.cargo\bin\cargo.exe test --test ui_input_error_tests
C:\Users\25994\.cargo\bin\cargo.exe test --example platform_input_smoke
C:\Users\25994\.cargo\bin\cargo.exe check --all-targets
npm.cmd run build
```

退出标准：

- 跨平台 trait 在 macOS / Windows CI 都会编译；noop fallback 由本地测试覆盖。
- `AppContext` 持有 `Box<dyn InputPlatform>`，前端 Permissions Tab 状态来自真实查询。
- 前端可通过 `getScreenTopology()` 获取后端屏幕拓扑 JSON。

### Sprint S2：mDNS 与 UDP 发现（已完成，手动双机 QA 待勾选）

目标：让两台设备在同 LAN 内 3 秒内互相出现。

任务：

- [x] `src-tauri/src/discovery/mdns.rs`：用 `mdns-sd = "0.13"`，发布 `_mac22win._tcp.local.`，TXT 记录按 00 §1.8.1：

  ```text
  device_id, device_name, os, arch, app_version, protocol_version, pairing
  ```

- [x] 浏览同服务，回调将 `ServiceEvent::ServiceResolved` 转成 `DiscoveredPeer` 并发到 `mpsc::Sender<DiscoveredPeer>`，由 `AppContext` 写入 `DiscoveryCache`。
- [x] 过滤本机：用 `device_id == local_identity.device_id` 比对，不要靠 IP。
- [x] `udp.rs`：监听 `0.0.0.0:42425` 接收广播，每 1.5 秒向 `255.255.255.255:42425` 发一帧 JSON：

  ```json
  {"v":1,"id":"...","name":"...","os":"macos","port":42424}
  ```

- [x] `discovery::mod` 添加 `start_discovery_tasks(...)`，把 mDNS + UDP 合并到同一个 discovery channel，由 `AppContext` 写入同一个 `DiscoveryCache`。
- [x] 后端发出 `device:discovered`、`device:stale` 两类事件，`useAppStatus` 订阅后刷新（参考 §5.3）。

改动文件：

```text
src-tauri/src/discovery/mdns.rs
src-tauri/src/discovery/udp.rs
src-tauri/src/discovery/mod.rs
src-tauri/src/discovery/cache.rs       (加 evict_stale + 时间戳更新)
src-tauri/src/app/context.rs           (后台任务装配)
src-tauri/Cargo.toml
src-tauri/examples/discovery_smoke.rs
src/hooks/useAppStatus.ts              (订阅事件)
src/app/tauri.ts                       (新增 listenDiscoveryEvents)
scripts/discovery-smoke.sh
scripts/discovery-smoke.ps1
```

新增依赖：`mdns-sd = "0.13"`、`socket2 = "0.5"`、`if-addrs = "0.13"`。

验证脚本（`scripts/discovery-smoke.sh`）：

```bash
# host A
./scripts/discovery-smoke.sh 3
# host B（另一台机器）
./scripts/discovery-smoke.sh 3
# 期望：3s 内双方日志都出现对端 device_id
```

Windows PowerShell：

```powershell
.\scripts\discovery-smoke.ps1 -Seconds 3
```

退出标准：

- 同 LAN 下 3s 内双向可见。
- 单方关闭 10s 内对方 list 中标记 stale 并消失。
- 路由器禁 multicast 时 UDP fallback 仍可见（手动测：在 macOS 关闭 mDNS 接口可以模拟）。
- `DiscoveryCache::list()` 返回空时前端 Devices Tab 显示 `t.devices.empty`。

### Sprint S3：配对协议与可信设备（3 天）

目标：6 位配对码 + 双端确认 + trust 持久化。

任务：

- `src-tauri/src/identity/keys.rs` 用 `ed25519-dalek = "2"` 生成真密钥对：

  ```rust
  pub struct DeviceKeypair {
      pub signing: ed25519_dalek::SigningKey,
      pub verifying: ed25519_dalek::VerifyingKey,
  }
  ```

- `storage/secret.rs` 抽象 `SecretStore` trait，V1 文件后端：私钥存到 `<config_dir>/identity.key`，文件权限 0600（macOS）/ ACL Owner-only（Windows，用 `windows::Win32::Security::Authorization`）。
- `pairing/flow.rs` 实质化为 `PairingState` 状态机：

  ```rust
  pub enum PairingState {
      Idle,
      OutgoingRequested { id, peer, code, expires_at },
      OutgoingAwaitingRemoteConfirm { id, peer, code },
      OutgoingAwaitingLocalConfirm { id, peer, code },
      IncomingRequest { id, peer, code, expires_at },
      Paired { peer_id, public_key },
      Rejected { reason },
      Expired,
  }
  ```

- 协议消息序列化：`PairingRequest / PairingResponse / PairingConfirm / PairingReject` 用 `bincode = "1.3"` payload，frame 类型按 §00 §5.4。
- `start_pairing(device_id)` 命令：发送 `PairingRequest`，状态变 `OutgoingRequested`，120s 后自动 expire，返回 `pairing_id` 给前端。
- `confirm_pairing(pairing_id)` 命令：发 `PairingConfirm`，写入 `trusted_peers.json`，emit `pairing:updated`。
- `reject_pairing(pairing_id)` 新增命令。
- 前端：新增 `PairingDialog` 组件，`pairing:request` 事件触发弹窗，显示 6 位码、远端设备名/OS/IP，两个按钮 confirm / reject。

改动文件：

```text
src-tauri/src/identity/keys.rs
src-tauri/src/storage/secret.rs
src-tauri/src/pairing/flow.rs
src-tauri/src/pairing/mod.rs
src-tauri/src/protocol/messages.rs       (PairingRequestBody / ResponseBody / ConfirmBody)
src-tauri/src/ui_api/commands.rs         (实质化 start_pairing / confirm_pairing / 新增 reject_pairing)
src-tauri/src/ui_api/events.rs           (定义 pairing:request / pairing:updated)
src-tauri/Cargo.toml
src/components/PairingDialog.tsx
src/hooks/usePairing.ts                  (new)
src/app/App.tsx                          (装配 PairingDialog)
```

新增依赖：`ed25519-dalek = "2"`、`bincode = "1.3"`、`zeroize = "1"`（私钥 drop）。

验证：

- `cargo test --test pairing_flow_tests`：覆盖 happy path、reject、expiry、双端 nonce 不一致。
- 手动两机：A 点击发现的 B 发起配对 → 两端弹出同一 6 位码 → 都点 confirm → A、B 的 `trusted_peers.json` 同时多出对方记录，公钥一致。

退出标准：

- 配对成功后重启 A、B，trust 仍在。
- A reject 时 B 显示 `pairing:rejected`，trust 不写入。
- 120 秒后未确认自动过期。
- 已配对设备的 `Hello.public_key` 与 pinned key 不符时，连接被立刻关闭并写入 `tracing::warn!` 日志。

### Sprint S4：TCP + Noise + 心跳 + 重连（5 天）

目标：可信对端之间的双向 Noise 加密 TCP 通道，掉线自愈。

#### S4.1 TCP 监听 + 客户端（1 天）

任务：

- `network/listener.rs`：`async fn serve(addr, ctx)` 用 `tokio::net::TcpListener`，每个连接 `set_nodelay(true)`，spawn `Connection::run`。
- `network/connector.rs`：`async fn connect(peer, ctx)`，连接超时 3s，连上后做 `Hello` 握手。
- `network/mod.rs` 增加 `Connection`，包装 `(read_half, write_half)`，对外暴露 `tx: mpsc::Sender<OutboundMessage>` 与 `rx: mpsc::Receiver<InboundMessage>`。
- `app::context::AppContext::start_network()`：监听端口、对每个 trusted peer 起一个 connector 任务。

#### S4.2 Noise 集成（2 天）

任务：

- 新增 `src-tauri/src/network/noise.rs`：
  - `NoiseInitiator::new(local_static_priv, remote_static_pub_pinned)` 用 `snow::Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2s")`。
  - `NoiseResponder::new(local_static_priv)`，收到第一帧后取出 ephemeral，第二帧后能拿到对端 static pub，与 trust 表比对，不匹配立刻关。
  - 握手完成后转 `TransportState`，`encrypt(plain) -> cipher`、`decrypt(cipher) -> plain`，每条 `MessageType` payload 单独加密。
- `network::framing` 不需要改，长度前缀外层不变；frame body（即 `header + payload`）作为整体经 Noise 加密。

  注意：现有 `encode_frame` 把 `length = HEADER_LEN + payload_len` 写到外面，加密后要把 `length` 改成 `cipher.len()`。最干净的做法是把 `framing` 拆两层：

  - `inner_frame` = `header + payload`（对应当前 `encode_frame` 的逻辑，去掉 length 前缀）。
  - `outer_frame` = `length u32 BE + cipher`。

  落地时：

  - `inner_encode(frame) -> Vec<u8>` 长度 = `HEADER_LEN + payload`。
  - `outer_encode(cipher) -> Vec<u8>` 长度 = `4 + cipher.len()`。
  - 接收端 `read_exact(4) -> length`，再 `read_exact(length)` 拿到密文，`noise.decrypt -> inner`，再 `inner_decode -> Frame`。

#### S4.3 心跳与重连（1.5 天）

任务：

- `network/heartbeat.rs` 实质化：
  - 发送侧每 1000ms 发一个 `HeartbeatPing { ping_id, sent_at_ms }`，记录 inflight。
  - 收侧立刻回 `HeartbeatPong`，原值带回。
  - 3000ms 内没收到 pong，关闭连接并触发重连。
- `network/reconnect.rs` 已有 `next_backoff_ms`，在外面包一层 `ReconnectPolicy`：
  - 加 0..250ms jitter（用 `rand::thread_rng().gen_range`）。
  - 稳定连接 30s 后 reset attempt 为 0。

#### S4.4 错误码与协议测试（0.5 天）

- 在 `protocol::messages` 增加：

  ```rust
  pub enum ProtocolError {
      UnsupportedProtocol { local: u16, remote: u16 },
      UnknownPeer { device_id: String },
      KeyMismatch { expected_fingerprint: String },
      HandshakeFailed,
      Internal(String),
  }
  ```

- 任何错误都通过 `MessageType::Error` 发一帧再 `Goodbye` 关闭，前端事件 `network:error` 显示。

改动文件：

```text
src-tauri/src/network/listener.rs
src-tauri/src/network/connector.rs
src-tauri/src/network/heartbeat.rs
src-tauri/src/network/reconnect.rs
src-tauri/src/network/noise.rs           (new)
src-tauri/src/network/framing.rs         (拆 inner/outer)
src-tauri/src/protocol/frame.rs          (length u32 字段语义注释)
src-tauri/src/protocol/messages.rs       (ProtocolError + Hello.noise_pattern)
src-tauri/src/network/mod.rs             (Connection)
src-tauri/Cargo.toml
src-tauri/tests/noise_handshake_tests.rs (new)
src-tauri/tests/heartbeat_tests.rs       (new)
```

新增依赖：`snow = "0.9"`、`rand = "0.8"`（已在）、`tokio = { features = ["net", "io-util"] }`。

验证：

- `cargo test --test noise_handshake_tests` 模拟 initiator + responder loopback，握手成功 + 解密正确。
- 拔网线测试：30s 内自动重连成功，前端 `session.state` 走 `ConnectedIdle → Reconnecting → ConnectedIdle`。
- 用 `tcpdump -i lo0 -X port 42424` 抓包，验证 Hello 之后所有 byte 不是明文 JSON。

退出标准：

- 已配对设备启动后自动建连。
- Noise 握手失败连接立即关，错误事件可见。
- 心跳 RTT 显示在前端诊断面板。
- 3 次内重连成功，30 分钟连续运行无 leak（用 `dtrace`/`Get-Process | Select WS` 观察）。

### Sprint S5：屏幕拓扑与布局编辑器（2 天）

目标：用户能在 UI 上拖出布局，运行时把布局对应到边缘检测。

任务：

- `ScreenTopology` 在 S1 已经能从 `InputPlatform::screen_topology()` 拿到，并已暴露 Tauri 命令 `get_screen_topology() -> UiScreenTopology`。
- 前端 `LayoutEditor` 重写：
  - Canvas 上画两块矩形（本机 + 远端）。
  - 可拖拽吸附到 `left/right/top/bottom`，松手后调 `save_layout`。
  - 当前活跃边缘高亮（绿色）。
  - 显示 `corner_guard_px` 实际覆盖的角落范围（半透明红块）。
- `input::edge` 已经实现，加 `pub fn detect_edge_hit(layout, topology, pointer) -> Option<EdgeHit>`，返回命中边和"剩余 delta（用于把超过边缘的部分作为远端 ControlEnter 的初始 delta）"。

改动文件：

```text
src-tauri/src/input/edge.rs              (detect_edge_hit + 测试)
src-tauri/src/ui_api/commands.rs         (get_screen_topology)
src-tauri/src/ui_api/models.rs           (UiScreenTopology)
src/components/LayoutEditor.tsx
src/app/types.ts
src-tauri/tests/edge_tests.rs            (new)
```

退出标准：

- 单/多显示器配置下，鼠标到达任意一块显示器的对应物理边缘都能触发。
- 角落保护：在 corner_guard 区域内不触发，远离角落时正常触发。
- 布局保存后重启依然生效。

### Sprint S6：鼠标无缝移交端到端（5 天）

目标：把 S1/S4/S5 串起来，做到 macOS ↔ Windows 真实跨设备控制。

任务：

- `session/controller.rs` 实质化为 `SessionController`：
  - 持有 `peer_id, control_owner, layout, topology`。
  - 监听 `mpsc::Receiver<LocalMouseEvent>`：
    - 本机为 owner 且未远控时，仅做边缘检测；命中则切到 `ControllingRemote`，向远端发 `ControlEnter`。
    - 远控期间，把所有事件转成 `RemoteMouseEvent` 发给 `network::Connection.tx`。
  - 监听 `mpsc::Receiver<RemoteMouseEvent>`（被控端）：
    - 收到 `ControlEnter` 切到 `ControlledByRemote`，warp 光标到入口位置。
    - 收到 `MouseMove` 调 `inject(MoveDelta)`。
    - 收到 `MouseButton/Wheel` 调对应注入。
    - 收到 `ControlLeave` 回到 `ConnectedIdle`。
- 反向边缘：远端在 `ControlledByRemote` 模式下检测对应边缘，命中则发 `ControlLeave` 把控制权还回主控端。
- `MouseMove` 合并：发送侧 channel 容量 256，满时把队列里相邻的 `Move` 累加 dx/dy 后丢弃前一个，但 `Button/Wheel` 永远独占一个槽，绝不合并。
- 紧急停止：
  - Tauri 全局快捷键 `CmdOrCtrl+Shift+Esc` 触发 `disconnect()`。
  - 系统托盘菜单"立即停止"。
- 自注入过滤：S1 已有标记，这里只做组合验证。

改动文件：

```text
src-tauri/src/session/controller.rs       (大改)
src-tauri/src/session/state.rs            (加 ControlEnter 时的 entry_edge / initial_position 字段)
src-tauri/src/protocol/messages.rs        (ControlEnter / ControlLeave / MouseMoveMsg / ButtonMsg / WheelMsg payload struct)
src-tauri/src/input/mod.rs                (PlatformInputManager 接到 SessionController)
src-tauri/src/app/context.rs              (装配后台任务)
src-tauri/src/ui_api/commands.rs          (start_control / stop_control)
src-tauri/tests/handoff_state_tests.rs    (new)
```

退出标准（手动 QA）：

- mac 控 win：左/右/上/下四个布局都能切，move/down/up/wheel 全部工作。
- win 控 mac：同上。
- 反向边缘返回：松手切回原机后本机鼠标继续工作，无卡死。
- 紧急停止：快捷键能在 100ms 内强制本地拿回控制。
- 30 分钟连续在两端来回切换，无明显内存增长。

### Sprint S7：稳定性、诊断与可见性（3 天）

目标：把 V1 推到能给少量用户内测的质量。

任务：

- `DiagnosticsPanel` 真接线：
  - 心跳 RTT、move fps、合并掉的 move 数、丢的事件数。
  - 当前 `SessionState` 时序图（最近 60s）。
  - 一键导出 `diagnostics.json` + `app.log` 最近 1MB 到桌面。
- 日志：
  - 所有 hook callback 和注入路径用 `tracing::trace!`，默认 `RUST_LOG=info`。
  - 错误用 `tracing::error!`，UI 弹 toast。
- 资源监控：内置 `interval(5s)` 采集 `process_memory` + `cpu_percent`，写到 `metrics`。
- 边缘场景：
  - 网线拔掉 → reconnecting，恢复后回 `ConnectedIdle` 不自动远控。
  - peer 退出 → 收到 FIN，状态回 `Paired`。
  - 权限被关 → 监听端关闭，`SessionState::Error("permission_revoked")`，UI 引导重新授权。
- 文档：把 `09-risk-assessment.md` 中"已缓解 / 待缓解"在 §6 更新一遍。

改动文件：

```text
src-tauri/src/telemetry/metrics.rs       (实质化采集)
src-tauri/src/telemetry/logging.rs       (export_recent_logs)
src-tauri/src/ui_api/commands.rs         (export_diagnostics)
src/components/DiagnosticsPanel.tsx
docs/09-risk-assessment.md               (status 字段更新)
```

退出标准：

- 30 分钟稳定性测试通过。
- 启动到 UI ready < 2s，rss < 200MB（macOS WebView）/ < 150MB（Windows WebView2）。
- 切换中位延迟 < 20ms（局域网 Wi-Fi）。
- 异常场景 UI 都有可读提示，没有"卡在 Loading 转圈"。

### Sprint S8：打包、签名与首发（3 天）

目标：可分发的 dmg / msi。

任务：

- macOS：
  - Apple Developer ID Application 证书（外部依赖，需账号）。
  - `tauri.conf.json` 配 `bundle.macos.signingIdentity`、`hardenedRuntime: true`、`entitlements`。
  - `notarytool submit` 公证并 staple。
- Windows：
  - 用 EV 或 OV 代码签名证书签 `flowlink.exe` 与 `flowlink.msi`。
  - 防火墙规则：安装器后置脚本 `netsh advfirewall firewall add rule` 放行 TCP 42424、UDP 42425。
- GitHub Actions：在 main 上打 tag `v0.1.0` 触发 release workflow，产出 `.dmg`、`.msi`，自动上传到 Release。
- README、CHANGELOG、Release Note 模板。

退出标准：

- 在新 mac 与新 win 上下载 release 产物，双击安装，按引导授权后能完成端到端 mouse handoff。

---

## 5. 跨 Sprint 的支撑工作

### 5.1 协议补丁：把 length 与 header 的关系写死

针对当前 `framing.rs` 与 00 §5.3 的口径差，正式约定如下：

```text
+-------------------------------------------------+
| length u32 BE      （= header_len + payload_len）|
+-------------------------------------------------+
| header u80         (version u16 + type u16 +    |
|                     flags u16 + seq u32)        |
+-------------------------------------------------+
| payload bytes                                   |
+-------------------------------------------------+
```

- `length` 不计自身的 4 字节。
- `MAX_FRAME_LENGTH = 65_536` 指 `length` 字段所表示的最大值。
- 经 Noise 加密后，`length` 字段表达的是密文长度（含 16 字节 Poly1305 标签），明文 frame 必须 `≤ MAX_FRAME_LENGTH - 16`。

### 5.2 鼠标按钮 wire 编码

目前 `MouseButton::Other(u8)` 是 Rust enum 带数据，`bincode` 默认会把判别字节写出来，远端反序列化无歧义。但为防 `bincode` 版本变更引发兼容性问题，wire format 强制按下面 1 字节编码：

```text
0x01 Left
0x02 Right
0x03 Middle
0x04 Back
0x05 Forward
0xFF Other(u8)   随后再读 1 字节
其他 reserved
```

`MouseButtonMsg` 的 `button` 字段在 wire 上是 1~2 字节，不直接复用 `bincode(MouseButton)`。

### 5.3 前端事件订阅

当前 `useAppStatus` 只在挂载时调一次 `get_app_status`。S2 之后改为 push 模型：

```ts
useEffect(() => {
  const unlistens = [
    listen("device:discovered", reload),
    listen("device:stale", reload),
    listen("session:state", reload),
    listen("permission:updated", reload),
    listen("pairing:request", showPairingDialog),
    listen("network:metrics", updateMetrics),
  ];
  return () => unlistens.forEach((u) => u.then((fn) => fn()));
}, []);
```

后端发事件用 `tauri::AppHandle::emit`，避免每个 Tab 都轮询。

### 5.4 性能基线

每个 Sprint 结束在 `bench/` 跑一遍，把结果写到 `bench/results/<sprint>.md`：

| 指标 | 目标 | 工具 |
| --- | --- | --- |
| 启动到 UI ready | < 2s | `start = now()` 在 `lib.rs::run` 入口；UI ready 用 `getCurrentWindow().show()` 时刻 |
| capture → 远端 inject 中位延迟 | < 20ms（LAN） | spike 二进制 + 双机 NTP 同步时钟 |
| 心跳 RTT P95 | < 5ms（LAN） | `network::heartbeat` 内部 histogram |
| 远控期间 CPU | < 8% | `top -pid` / `Get-Counter` |
| 稳态 RSS | < 200MB（含 WebView） | `ps -o rss` |
| 30 分钟内存增长 | < 5MB | 每分钟采样 |
| Move 合并率 | 50%~80%（高频抖动场景） | `metrics::moves_in / moves_out` |

### 5.5 QA Matrix 自动化

`bench/qa-matrix.toml` 列出所有手动测试组合，跑前 print 出来对着打勾：

```toml
[[combo]]
host_a = "macOS Apple Silicon"
host_b = "Windows 11"
network = "wired"
layout = "right"
permission = "all_granted"

# ... 共 4 设备对 × 3 网络 × 4 布局 × 4 权限 = 192 个组合
# V1 至少跑 20 个采样组合
```

### 5.6 文档同步

每个 Sprint 完成时，要更新：

- `08-mvp-roadmap.md` 对应 Phase 改成 `(done <date>)`。
- 本文 §2.1 / §2.2 把已完成项移到上方。
- 如果发现协议或数据结构与 00/04/05 不一致，先改设计文档再改代码，不允许"代码先跑设计后补"。
- README "当前状态" 段每个 Sprint 末刷新一次。

---

## 6. 风险状态更新（对照 09）

| 风险 | 09 中状态 | 现在状态 | 缓解归属 |
| --- | --- | --- | --- |
| macOS 权限失败 | 高 | 高（未实现检测）| S1 |
| 光标抑制不稳 | 中 | 中（V1 不抑制）| 文档说明 + S6 验证 |
| Windows 控不了管理员窗口 | 中 | 中 | S7 文案 + V2 elevated helper |
| mDNS 被网络阻断 | 高 | 中（mDNS + UDP fallback 已实现，仍需双机网络 QA）| S2 / P059 smoke + IP 直连 |
| Wi-Fi 抖动 | 中 | 待 S4 后实测 | S4 + S6 |
| 自注入反馈循环 | 中 | 中（仅设计，未实现标记）| S1.1/S1.2 |
| DPI / Retina 误差 | 中 | 中 | S5 |
| 未授权设备尝试连 | 中 | 高（当前完全无握手）| S3 + S4 Noise |
| 私钥被读 | 09 未列 | 高 | S3 文件权限 + secret store |
| 截断帧 panic | 09 未列 | 已缓解（`Incomplete` 测试） | S0 单测 |

新增风险：

- **S4 Noise 集成出错可能导致死锁**：握手期间双方都在 `read` 时容易卡住。缓解：握手用 `tokio::time::timeout(Duration::from_secs(5), ...)`，超时即关连接。
- **S6 鼠标合并把 Click + Drag 拆错**：Drag 的"Down → Move × N → Up"序列里 Move 合并时不能把 Down/Up 跨过。实现层面：合并仅在 channel 末尾相邻两个 Move 之间发生，遇到任何非 Move 立即停止合并。

---

## 7. V1.x → V2 衔接建议

V1 收尾时把以下三件事做完，能让 10 中规划的 V2 演进顺手：

1. **Trait 边界清晰**：`InputPlatform`、`SecretStore`、`Transport` 这三个抽象 V1 就抽好；V2 加键盘共享 / 加 QUIC / 加 Keychain 后端时只换实现。
2. **协议预留版本字段**：`Hello.protocol_version` 在 V1 锁 1，V1.1 升 2 时同时支持 1/2 双向。
3. **Daemon 化预演**：在 S6 把 `SessionController` 写成 `tokio::Runtime::block_on` 顶层任务，不依赖 Tauri lifecycle，V2 抽 daemon 时可以原样搬。

不在 V1 做、但建议 V1.x 优先级排序：

```text
V1.1  托盘 + 全局快捷键紧急停止 + Windows 安装器写防火墙规则
V1.2  自动更新 (Tauri Updater + 签名校验)
V1.3  剪贴板文本同步（默认关，明确开关，限制 1 MiB）
V1.4  键盘共享（仅基础键，不含 IME 高级场景）
V1.5  多显示器拓扑可视化（一台机器多屏的边缘细化）
V2.0  三台及以上设备布局 + Daemon 化
```

---

## 8. 第一周可立刻动手的事项

如果只看时间最近的下一步：

1. 把本文存档（已完成）。
2. 跑 Sprint S0 的测试与 CI（1.5 天）。
3. 起 Sprint S1.1 macOS Spike，先跑通 `examples/mac_input_spike.rs`（不要 Tauri 集成，避开权限身份变化的坑），等"能监听 + 能注入 + 自标记过滤"三件事都对了，再合到 `MacInputPlatform` 里。

到这一步可以决定：

- 如果 macOS 权限链路顺，按 S2/S3/S4 并行推进。
- 如果 macOS 权限非常折腾（dev build 不能持久授权 / Tauri reload 后权限失效等），先做 S8 中"开发证书签 dev build"那一段，再回来推 Spike。

---

## 9. 暂未决定的开放问题

下列问题需要在对应 Sprint 开始前决定，或继续讨论：

- **配对码展示位置**：是只在主控端弹窗，还是被动端也弹？00 §1.7.3 写"两端都弹"，落地时若被动端尚未在前端，至少要在 tracing log 中打出。
- **远控期间本机光标处理**：00 §2.8 已写"停在边缘内侧 1~2 px"。是否在 V1 尝试 `CGAssociateMouseAndMouseCursorPosition(false)`（macOS）和 `ClipCursor`（Windows）做更彻底的抑制？建议作为 S6 内部 spike，不通过则保守回退。
- **持久化加密**：`identity.key` 是否需要本机口令解密？V1 建议不强制，V1.1 引入 SecretStore 后再做。
- **公网穿透时机**：10 中放 V3，但若内测反馈"很多用户在两个不同 Wi-Fi 段"，可能要前移；目前不做。
- **多设备同时活跃**：00 §2.2 写"V1 同一时间只允许一个活跃 peer"。如果用户配了 A、B 两台 trusted peer，UI 上由用户手动切换 active，还是按"上次连接的那个"自动选？建议 S6 中用 `last_selected_peer_id` 自动选，UI 提供下拉切换。

---

## 10. 文档维护

本篇按 Sprint 节奏迭代：

- Sprint 完成时更新对应 §4 段落，把任务列表打勾或挪走。
- §2 现状清单跟着代码同步。
- 与 00/04/05 出现冲突时，开一个章节注明，并在下一版（v0.3）合并到主文档。

下一版预期触发：S2 完成后，因为届时协议 wire 第一次真实跑通，需要把 Hello / Discovery / Pairing 的实际字段与 00/04/05 对齐。



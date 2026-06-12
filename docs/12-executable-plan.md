# FlowLink 可执行任务索引

文档版本：`v0.1`

创建时间：2026-06-11

用途：把 `docs/11-next-steps.md` 拆成稳定编号的可执行任务。后续协作时，用户可以直接说“做 P042”，Codex 就按对应任务实施、验证、审查。

## 当前进度快照

- S0：已有测试、CI、日志、坏 JSON 恢复等实现痕迹，但当前 `cargo test` 失败，直接原因是集成测试仍引用 `flowlink::...`，而当前库名是 `flowlink_lib`。
- S1.1：macOS 输入监听/注入已有初版，权限检测/打开设置也已有实现，但仍需要 spike 验证、权限状态接线和实机验收。
- S1.2：Windows 输入监听/注入已有初版，包含 Hook、SendInput、DPI、显示器枚举雏形，但需要编译修正、Windows 实机验证和管理员窗口限制提示。
- S1.3：`InputPlatform` trait 已存在，`AppContext` 已持有 `Box<dyn InputPlatform>`，但前端权限状态仍需真实刷新闭环。
- S2-S8：大部分仍是待实现，尤其是 mDNS/UDP、Pairing、TCP+Noise、SessionController、LayoutEditor、Diagnostics、Packaging。
- UI 图标：前端与 Tauri 图标资源已处理过一轮；如后续仍有 Dock/Finder 缓存问题，单独按 P120 处理。

## 使用规则

- 每个 P 代表一个可独立完成的小任务，默认包含实现、最小测试、一次代码审查。
- 除非特别说明，完成一个 P 后至少运行与该 P 相关的最小验证命令。
- 如果任务触及平台能力，macOS 任务在当前机器验证；Windows 任务至少保证交叉编译/静态检查，实机验证需在 Windows 环境执行。
- 如果发现 `11-next-steps.md` 与代码冲突，先在对应 P 中更新文档，再改代码。

## P001-P010：立即恢复基线

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P001 | 修复 Rust crate 名称不一致，统一集成测试和 examples 的 `use flowlink...` / `use flowlink_lib...`。 | `src-tauri/Cargo.toml`, `src-tauri/tests/*`, `src-tauri/examples/*`, `src-tauri/src/main.rs` | `cd src-tauri && cargo test` 不再因 unresolved crate 失败。 |
| P002 | 跑通并修复 `cargo test` 的所有现有失败。 | `src-tauri/src/**`, `src-tauri/tests/**` | `cd src-tauri && cargo test` 全绿。 |
| P003 | 跑通并修复 `cargo clippy --all-targets -- -D warnings`。 | `src-tauri/src/**`, `src-tauri/tests/**`, `src-tauri/examples/**` | clippy 全绿且不通过 `allow` 掩盖真实问题。 |
| P004 | 跑通并修复 `cargo fmt --check`。 | Rust 全部文件 | `cd src-tauri && cargo fmt --check` 全绿。 |
| P005 | 跑通并修复前端构建。 | `src/**`, `package.json`, `vite.config.*` | 仓库根目录 `npm run build` 全绿。 |
| P006 | 审查并修正 CI 与本地命令一致性。 | `.github/workflows/ci.yml` | CI 命令包含 fmt、clippy、test、npm build，工作目录正确。 |
| P007 | 检查启动时 demo peer 是否彻底移除。 | `src-tauri/src/app/context.rs`, `src-tauri/src/discovery/**`, `src/app/tauri.ts` | 真应用启动 `discoveredDevices` 为空；浏览器 mock 仍可展示 demo。 |
| P008 | 验证坏 JSON 恢复行为并补缺失测试。 | `src-tauri/src/storage/files.rs`, `src-tauri/tests/storage_tests.rs` | 坏 JSON 生成 `.corrupt.<timestamp>`，返回并写入默认值。 |
| P009 | 验证日志按天滚动写入真实 app log dir。 | `src-tauri/src/telemetry/logging.rs`, `src-tauri/src/lib.rs` | 启动后本地日志目录出现 `app.log`，重复 init 不 panic。 |
| P010 | 更新 `docs/11-next-steps.md` 的 S0 当前状态。 | `docs/11-next-steps.md` | S0 已完成/未完成项与当前代码一致。 |

## P011-P025：S1.1 macOS 监听与注入

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P011 | 审查 `MacInputPlatform::permissions()`，改为使用真实 macOS 权限查询，不再基于临时 identity。 | `src-tauri/src/input/macos.rs`, `src-tauri/src/platform/macos_permissions.rs` | Permissions Tab 显示 Accessibility/Input Monitoring 真实状态。 |
| P012 | 修正 `open_permission_settings` 的 macOS 设置 URL，覆盖 Accessibility 与 Input Monitoring。 | `src-tauri/src/platform/macos_permissions.rs`, `src-tauri/src/ui_api/commands.rs` | 点击按钮能打开系统设置对应隐私页。 |
| P013 | 增加 macOS 权限刷新事件或刷新命令闭环。 | `src-tauri/src/ui_api/commands.rs`, `src/hooks/useAppStatus.ts` | 授权后点击刷新或回到应用，UI 状态更新。 |
| P014 | 完善 `examples/mac_input_spike.rs`，打印 move/down/up/wheel 事件。 | `src-tauri/examples/mac_input_spike.rs` | 授权后运行 spike 能采集 1000 个事件。 |
| P015 | 为 macOS spike 增加注入移动测试。 | `src-tauri/examples/mac_input_spike.rs`, `src-tauri/src/input/macos.rs` | spike 能注入 100 次小幅移动并输出耗时。 |
| P016 | 为 macOS spike 增加点击注入测试。 | `src-tauri/examples/mac_input_spike.rs`, `src-tauri/src/input/macos.rs` | TextEdit 或桌面可响应一次真实点击。 |
| P017 | 验证并修复 macOS 自注入过滤标记。 | `src-tauri/src/input/macos.rs`, `src-tauri/examples/mac_input_spike.rs` | 注入事件不会再次进入本地 capture 队列。 |
| P018 | 完善 macOS scroll wheel 的 dx/dy 方向与单位。 | `src-tauri/src/input/macos.rs` | 纵向/横向滚轮方向与系统一致。 |
| P019 | 完善 macOS other mouse button 映射。 | `src-tauri/src/input/macos.rs`, `src-tauri/src/protocol/messages.rs` | 中键、侧键不会误映射为左键。 |
| P020 | 验证 macOS `screen_topology()` 的多显示器 bounds 与 scale。 | `src-tauri/src/input/macos.rs`, `src-tauri/examples/mac_input_spike.rs` | spike 输出所有显示器、主屏、scale_factor。 |
| P021 | 为 macOS capture handle 设计最小关闭机制。 | `src-tauri/src/input/types.rs`, `src-tauri/src/input/macos.rs` | 测试/退出时不会遗留不可控 capture 线程。 |
| P022 | macOS 输入错误类型细化。 | `src-tauri/src/input/types.rs`, `src-tauri/src/input/macos.rs` | 权限失败、EventTap 失败、注入失败错误可区分。 |
| P023 | macOS 权限文案同步到中英文 i18n。 | `src/app/i18n.ts`, `src/app/App.tsx` | UI 提示用户需要 Accessibility 和 Input Monitoring。 |
| P024 | macOS S1.1 最小 QA 记录。 | `bench/results/s1.1-macos.md` | 记录监听、注入、自过滤、延迟结果。 |
| P025 | 更新文档中 S1.1 完成状态和已知限制。 | `docs/11-next-steps.md`, `docs/09-risk-assessment.md` | macOS 权限风险状态被更新。 |

## P026-P040：S1.2 Windows 监听与注入

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P026 | 修复 Windows 代码在非 Windows 主机上的条件编译边界。 | `src-tauri/src/input/windows.rs`, `src-tauri/Cargo.toml` | macOS 上 `cargo test --all-targets` 不因 Windows 代码失败。 |
| P027 | 在 Windows 环境跑通 `cargo check` 并修复 windows crate API 差异。 | `src-tauri/src/input/windows.rs`, `src-tauri/Cargo.toml` | Windows 上 `cargo check --all-targets` 全绿。 |
| P028 | 完善 `examples/win_input_spike.rs` 事件打印。 | `src-tauri/examples/win_input_spike.rs` | Windows 上能打印 1000 个 mouse hook 事件。 |
| P029 | 完善 Windows SendInput 移动注入测试。 | `src-tauri/src/input/windows.rs`, `src-tauri/examples/win_input_spike.rs` | Windows 桌面可看到 100 次平移。 |
| P030 | 完善 Windows 点击注入。 | `src-tauri/src/input/windows.rs`, `src-tauri/examples/win_input_spike.rs` | 普通窗口可响应 left/right/middle click。 |
| P031 | 完善 Windows wheel/hwheel 注入方向。 | `src-tauri/src/input/windows.rs` | 垂直/水平滚动方向符合 Windows 习惯。 |
| P032 | 修复 Windows Back/Forward 侧键 SendInput 映射。 | `src-tauri/src/input/windows.rs` | XBUTTON1/XBUTTON2 使用正确 `mouseData` 和 flags。 |
| P033 | 验证 Windows 自注入过滤 `dwExtraInfo`。 | `src-tauri/src/input/windows.rs`, `src-tauri/examples/win_input_spike.rs` | SendInput 事件不会被 hook 当作本地事件上报。 |
| P034 | 完善 Windows DPI awareness 初始化时机。 | `src-tauri/src/input/windows.rs`, `src-tauri/src/lib.rs` | 进程启动早期设置 Per Monitor V2，不重复报错。 |
| P035 | 验证 Windows 多显示器 absolute 坐标归一化。 | `src-tauri/src/input/windows.rs`, `src-tauri/examples/win_input_spike.rs` | 多屏绝对定位误差 ≤ 1 像素。 |
| P036 | 实现 Windows integrity level 检测。 | `src-tauri/src/platform/windows_permissions.rs` | UI 能提示普通/管理员窗口限制。 |
| P037 | Windows 权限/限制文案接入前端。 | `src/app/i18n.ts`, `src/app/App.tsx`, `src-tauri/src/ui_api/models.rs` | Permissions Tab 显示 Windows 输入限制说明。 |
| P038 | Windows hook 生命周期关闭机制。 | `src-tauri/src/input/windows.rs`, `src-tauri/src/input/types.rs` | 关闭 capture 后 hook 释放，线程退出。 |
| P039 | Windows S1.2 最小 QA 记录。 | `bench/results/s1.2-windows.md` | 记录 Win10/Win11 监听、注入、自过滤结果。 |
| P040 | 更新文档中 S1.2 完成状态和已知限制。 | `docs/11-next-steps.md`, `docs/09-risk-assessment.md` | Windows 管理员窗口风险有说明。 |

## P041-P048：S1.3 平台抽象与前端接线

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P041 | 补齐 `NoopInputPlatform` 行为和测试。 | `src-tauri/src/input/noop.rs`, `src-tauri/tests/*` | Linux/CI 下返回 Unsupported，不 panic。 |
| P042 | 将 `PermissionStatus` 改为完全来自 `InputPlatform`。 | `src-tauri/src/app/context.rs`, `src-tauri/src/platform/mod.rs` | 不再用 identity 伪造权限状态。 |
| P043 | 新增 `get_screen_topology` Tauri 命令。 | `src-tauri/src/ui_api/commands.rs`, `src-tauri/src/ui_api/models.rs`, `src-tauri/src/lib.rs` | 前端可获取屏幕拓扑 JSON。 |
| P044 | 前端增加屏幕拓扑类型定义。 | `src/app/types.ts`, `src/app/tauri.ts` | TypeScript 编译通过。 |
| P045 | 输入平台错误映射到 `UiError`。 | `src-tauri/src/ui_api/commands.rs`, `src-tauri/src/ui_api/models.rs` | 权限/平台错误能显示 recoverable code。 |
| P046 | 增加 platform input smoke 测试入口。 | `src-tauri/examples/platform_input_smoke.rs` | macOS/Windows 能用同一入口跑监听/注入。 |
| P047 | 确认 S1 跨平台 CI 编译策略。 | `.github/workflows/ci.yml`, `src-tauri/Cargo.toml` | macOS/Windows runner 都跑 Rust checks。 |
| P048 | 更新 S1 总结文档。 | `docs/11-next-steps.md` | S1.1-S1.3 状态准确。 |

## P049-P060：S2 mDNS 与 UDP 发现

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P049 | 添加发现依赖并确认跨平台编译。 | `src-tauri/Cargo.toml` | `mdns-sd`, `socket2`, `if-addrs` 编译通过。 |
| P050 | 实现 mDNS 服务发布。 | `src-tauri/src/discovery/mdns.rs` | 发布 `_mac22win._tcp.local.`，TXT 字段完整。 |
| P051 | 实现 mDNS 浏览和解析。 | `src-tauri/src/discovery/mdns.rs` | 同 LAN 解析到 peer 地址、端口、TXT。 |
| P052 | 实现本机过滤。 | `src-tauri/src/discovery/mdns.rs`, `src-tauri/src/discovery/mod.rs` | `device_id == local` 的服务不会进入列表。 |
| P053 | 实现 UDP 广播发送。 | `src-tauri/src/discovery/udp.rs` | 每 1.5s 广播 JSON announce。 |
| P054 | 实现 UDP 广播监听。 | `src-tauri/src/discovery/udp.rs` | 能接收对端 announce 并转为 `DiscoveredPeer`。 |
| P055 | 合并 mDNS 与 UDP 到统一 discovery task。 | `src-tauri/src/discovery/mod.rs`, `src-tauri/src/app/context.rs` | 两种来源写入同一 `DiscoveryCache`。 |
| P056 | 完善 `DiscoveryCache::evict_stale`。 | `src-tauri/src/discovery/cache.rs`, `src-tauri/tests/*` | 10s 未刷新 peer 标记 stale 或移除。 |
| P057 | 后端 emit `device:discovered` / `device:stale`。 | `src-tauri/src/ui_api/events.rs`, `src-tauri/src/discovery/mod.rs` | 前端能收到事件。 |
| P058 | 前端订阅发现事件并刷新设备列表。 | `src/hooks/useAppStatus.ts`, `src/app/tauri.ts` | 两机发现后 UI 3s 内更新。 |
| P059 | 新增 discovery smoke 脚本。 | `scripts/discovery-smoke.sh` | 两台机器 3s 内互相发现。 |
| P060 | S2 文档和风险更新。 | `docs/11-next-steps.md`, `docs/09-risk-assessment.md` | mDNS 阻断风险有 UDP fallback 说明。 |

## P061-P075：S3 配对与可信设备

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P061 | 引入 Ed25519 依赖。 | `src-tauri/Cargo.toml` | `ed25519-dalek`, `zeroize` 编译通过。 |
| P062 | 实现真实设备密钥生成。 | `src-tauri/src/identity/keys.rs` | 生成 signing/verifying key，公钥稳定序列化。 |
| P063 | 实现文件 SecretStore trait。 | `src-tauri/src/storage/secret.rs` | 私钥可保存/读取。 |
| P064 | macOS 私钥文件权限 0600。 | `src-tauri/src/storage/secret.rs` | `stat` 显示 owner-only。 |
| P065 | Windows 私钥 owner-only ACL。 | `src-tauri/src/storage/secret.rs` | Windows 上非 owner 不能读取。 |
| P066 | 将 identity 加载接入真实 keypair。 | `src-tauri/src/app/context.rs`, `src-tauri/src/identity/**` | 重启后 device_id/public_key 稳定。 |
| P067 | 实现 PairingState 状态机。 | `src-tauri/src/pairing/flow.rs` | happy/reject/expire 单测通过。 |
| P068 | 定义 Pairing 消息 payload。 | `src-tauri/src/protocol/messages.rs` | bincode 序列化往返测试通过。 |
| P069 | 实现 `start_pairing(device_id)`。 | `src-tauri/src/ui_api/commands.rs`, `src-tauri/src/pairing/**` | 返回 pairing_id，并 emit 状态。 |
| P070 | 实现 `confirm_pairing(pairing_id)` 写 trust。 | `src-tauri/src/ui_api/commands.rs`, `src-tauri/src/storage/files.rs` | `trusted_peers.json` 写入对端公钥。 |
| P071 | 新增 `reject_pairing(pairing_id)`。 | `src-tauri/src/ui_api/commands.rs`, `src-tauri/src/lib.rs` | 对端收到 rejected，trust 不写入。 |
| P072 | 前端 PairingDialog 组件。 | `src/components/PairingDialog.tsx`, `src/app/App.tsx` | 两端显示同一 6 位码。 |
| P073 | 前端 `usePairing` 事件订阅。 | `src/hooks/usePairing.ts`, `src/app/tauri.ts` | `pairing:request/updated/rejected` 可驱动弹窗。 |
| P074 | 配对集成测试。 | `src-tauri/tests/pairing_flow_tests.rs` | happy/reject/expiry/key mismatch 测试通过。 |
| P075 | S3 文档更新。 | `docs/11-next-steps.md` | 开放问题“配对码展示位置”落定。 |

## P076-P092：S4 TCP + Noise + 心跳 + 重连

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P076 | 为 tokio 启用 net/io-util 必要 features。 | `src-tauri/Cargo.toml` | TcpListener/TcpStream 编译通过。 |
| P077 | 实现 TCP listener bind/accept loop。 | `src-tauri/src/network/listener.rs` | 本地端口 42424 可监听。 |
| P078 | 实现 TCP connector。 | `src-tauri/src/network/connector.rs` | 对 peer 地址 3s 超时连接。 |
| P079 | 定义 `Connection` 与 inbound/outbound channel。 | `src-tauri/src/network/mod.rs` | 可用 mpsc 发送协议消息。 |
| P080 | 实现明文 Hello/HelloAck。 | `src-tauri/src/protocol/messages.rs`, `src-tauri/src/network/**` | 协议版本和 device_id 能协商。 |
| P081 | 拆分 inner frame 与 outer frame。 | `src-tauri/src/network/framing.rs`, `src-tauri/src/protocol/frame.rs` | 现有 framing 测试更新后全绿。 |
| P082 | 引入 snow 并实现 Noise initiator。 | `src-tauri/Cargo.toml`, `src-tauri/src/network/noise.rs` | initiator 生成握手消息。 |
| P083 | 实现 Noise responder。 | `src-tauri/src/network/noise.rs` | responder 完成 XX 握手。 |
| P084 | Noise 握手绑定 pinned key。 | `src-tauri/src/network/noise.rs`, `src-tauri/src/config/mod.rs` | key mismatch 立刻断开。 |
| P085 | Noise transport encrypt/decrypt 接入 Connection。 | `src-tauri/src/network/mod.rs`, `src-tauri/src/network/noise.rs` | Hello 后 payload 不再明文。 |
| P086 | 增加 Noise loopback 测试。 | `src-tauri/tests/noise_handshake_tests.rs` | 握手成功、解密正确、key mismatch 失败。 |
| P087 | 实现 HeartbeatPing/Pong。 | `src-tauri/src/network/heartbeat.rs`, `src-tauri/src/protocol/messages.rs` | 1s ping，3s timeout。 |
| P088 | 实现 ReconnectPolicy jitter/reset。 | `src-tauri/src/network/reconnect.rs` | 稳定 30s 后 attempt reset。 |
| P089 | 后端 session/network 错误事件。 | `src-tauri/src/ui_api/events.rs`, `src-tauri/src/protocol/messages.rs` | UI 能看到 network:error。 |
| P090 | AppContext 启动网络后台任务。 | `src-tauri/src/app/context.rs`, `src-tauri/src/lib.rs` | trusted peer 自动连接。 |
| P091 | tcpdump 抓包验证脚本。 | `scripts/network-capture-smoke.sh` | Hello 后无明文 JSON。 |
| P092 | S4 文档更新。 | `docs/11-next-steps.md`, `docs/05-network-protocol.md` | length/header/Noise 口径一致。 |

## P093-P101：S5 屏幕拓扑与布局

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P093 | 实现 `detect_edge_hit`。 | `src-tauri/src/input/edge.rs` | 返回命中边和剩余 delta。 |
| P094 | 增加 edge 多屏测试。 | `src-tauri/tests/edge_tests.rs` | 单屏/多屏/角落保护全覆盖。 |
| P095 | 前端新增 `LayoutEditor` 替换静态 Canvas。 | `src/components/LayoutEditor.tsx`, `src/app/App.tsx` | 可拖拽设置上下左右布局。 |
| P096 | 保存布局时保留 edge/corner 配置。 | `src-tauri/src/ui_api/commands.rs`, `src/app/tauri.ts` | 重启后布局仍生效。 |
| P097 | UI 显示 corner_guard 区域。 | `src/components/LayoutEditor.tsx`, `src/styles/app.css` | 红色半透明角落区域可见。 |
| P098 | UI 高亮当前活跃边缘。 | `src/components/LayoutEditor.tsx` | 选中的边缘绿色高亮。 |
| P099 | 设备选择与布局绑定。 | `src/app/App.tsx`, `src/app/types.ts` | 可为特定 peer 保存 layout。 |
| P100 | 布局 smoke 测试文档。 | `bench/results/s5-layout.md` | 记录单屏、多屏、四方向验证。 |
| P101 | S5 文档更新。 | `docs/11-next-steps.md` | Layout 与 edge 状态准确。 |

## P102-P112：S6 鼠标无缝移交端到端

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P102 | 实现 `SessionController` 基础状态机。 | `src-tauri/src/session/controller.rs`, `src-tauri/src/session/state.rs` | ConnectedIdle/ControllingRemote/ControlledByRemote 可切换。 |
| P103 | 本地边缘触发 `ControlEnter`。 | `src-tauri/src/session/controller.rs`, `src-tauri/src/input/edge.rs` | 到边缘发送 ControlEnter。 |
| P104 | 被控端处理 `ControlEnter` 并 warp。 | `src-tauri/src/session/controller.rs`, `src-tauri/src/input/mod.rs` | 远端光标进入正确位置。 |
| P105 | MouseMove delta 发送与注入。 | `src-tauri/src/session/controller.rs`, `src-tauri/src/protocol/messages.rs` | 跨设备移动流畅。 |
| P106 | MouseButton/Wheel 发送与注入。 | 同上 | click/drag/wheel 跨设备正常。 |
| P107 | 实现反向边缘 `ControlLeave`。 | `src-tauri/src/session/controller.rs` | 从远端边缘返回本机。 |
| P108 | 实现 Move 合并队列。 | `src-tauri/src/session/controller.rs` | Move 可合并，Button/Wheel 不丢不乱序。 |
| P109 | 实现紧急停止命令。 | `src-tauri/src/ui_api/commands.rs`, `src-tauri/src/session/controller.rs` | UI 按钮 100ms 内停止远控。 |
| P110 | 增加全局快捷键紧急停止。 | `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs` | Cmd/Ctrl+Shift+Esc 生效。 |
| P111 | 增加 handoff 状态测试。 | `src-tauri/tests/handoff_state_tests.rs` | enter/move/button/leave/emergency 全覆盖。 |
| P112 | S6 双机 QA 记录。 | `bench/results/s6-handoff.md` | mac 控 win、win 控 mac、四方向、30 分钟记录。 |

## P113-P118：S7 稳定性、诊断与可见性

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P113 | 实现 metrics 采集。 | `src-tauri/src/telemetry/metrics.rs` | RTT、fps、drop、merge、CPU/RSS 有值。 |
| P114 | 实现 DiagnosticsPanel。 | `src/components/DiagnosticsPanel.tsx`, `src/app/App.tsx` | UI 可查看最近 60s 状态。 |
| P115 | 实现导出 diagnostics。 | `src-tauri/src/ui_api/commands.rs`, `src-tauri/src/telemetry/**` | 桌面生成 diagnostics.json + app.log。 |
| P116 | 网络异常场景处理。 | `src-tauri/src/network/**`, `src-tauri/src/session/**` | 拔网线后 reconnecting，恢复后 idle。 |
| P117 | 权限被撤销场景处理。 | `src-tauri/src/input/**`, `src-tauri/src/session/**`, `src/app/App.tsx` | UI 提示 permission_revoked。 |
| P118 | S7 30 分钟稳定性报告。 | `bench/results/s7-stability.md`, `docs/09-risk-assessment.md` | RSS 增长 < 5MB，异常场景有记录。 |

## P119-P120：S8 打包与发布准备

| ID | 任务 | 文件范围 | 验收 |
| --- | --- | --- | --- |
| P119 | 配置 macOS/Windows 基础 bundle icon 与 release workflow。 | `src-tauri/tauri.conf.json`, `.github/workflows/release.yml` | tag 后能产出 unsigned dmg/msi。 |
| P120 | 处理 macOS 图标缓存、签名、公证和最终安装包图标验证。 | `src-tauri/icons/**`, `src-tauri/tauri.conf.json`, release 产物 | 新安装 `.app` 在 Finder/Dock 显示圆角图标。 |

## 推荐推进顺序

1. 先做 P001-P005，把本地验证恢复为绿灯。
2. 然后做 P011-P025，把 macOS 端输入能力实测跑通。
3. 再做 P026-P040，在 Windows 端完成监听与注入实测。
4. S1 全绿后，按 P049-P092 推发现、配对、TCP+Noise。
5. 最后按 P093-P120 串 UI 布局、handoff、诊断和打包。

## 下一步建议

最建议下一个任务是 P001。当前测试基线因为 crate 名称不一致而失败，这会拖累后续所有实现和审查。P001 完成后再做 P002-P005，整个项目就重新有安全网了。

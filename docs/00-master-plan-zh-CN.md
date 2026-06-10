# 跨设备鼠标协同 MVP 产品与技术方案

文档版本：`v0.1`

目标平台：macOS、Windows

推荐技术栈：**Tauri + Rust + TypeScript**

V1 范围：局域网内两台设备自动发现、配对、布局配置、鼠标移动/点击/滚轮跨设备控制、状态同步、心跳、断线重连。

V1 不做：账号体系、公网穿透、云服务、文件传输、剪贴板同步、键盘同步、多设备网格、商业化能力。

## 0. 结论摘要

### 0.1 最终推荐方案

V1 MVP 推荐使用 **Tauri + Rust**。

推荐理由：

- 鼠标监听和输入模拟是本项目最大技术风险，Rust 更适合直接调用 macOS CoreGraphics/ApplicationServices 和 Windows Win32 API。
- Tauri 使用系统 WebView，不内置 Chromium，更容易接近内存 `< 100MB` 的目标。
- Rust 的网络、并发、二进制协议和状态机实现更稳，适合低延迟输入事件链路。
- Tauri 前端可以用 TypeScript 快速实现配置 UI，底层热路径完全留在 Rust，避免高频鼠标事件穿过 WebView。
- 后续可以把 Rust Core 抽成后台 daemon/service，Tauri 只保留控制面板，扩展性好。

### 0.2 V1 通信推荐

实时控制通道推荐 **TCP + 长连接 + 长度前缀二进制协议 + TCP_NODELAY**。

设备发现推荐 **mDNS/Zeroconf 为主，UDP Broadcast 为兜底，IP 直连为必备后备路径**。

### 0.3 V1 权限推荐

macOS：

- `Accessibility`：用于注入鼠标事件，也通常影响全局事件控制能力。
- `Input Monitoring`：用于监听全局鼠标输入。
- `Screen Recording`：V1 鼠标控制不需要，不应在 V1 首次启动时申请；未来如果做屏幕预览、截图、远程画面再申请。

Windows：

- 普通用户权限即可完成正常桌面应用的低级鼠标 Hook 和 `SendInput` 注入。
- 不要求管理员权限。
- 非管理员进程可能无法控制管理员权限窗口、UAC 安全桌面或受保护程序，这是 Windows 完整性级别和 UIPI 的限制。

### 0.4 官方 API 依据

- Apple `CGEvent.tapCreate` 用于创建事件 Tap，未获权限时可能返回空，并且 HID 入口级 Tap 对权限要求更高：https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate(tap:place:options:eventsofinterest:callback:userinfo:)
- Apple `CGEvent.post` 用于向事件流投递 Quartz 事件：https://developer.apple.com/documentation/coregraphics/cgevent/post(tap:)
- Apple `CGPreflightListenEventAccess` 用于检查输入监听权限：https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess()
- Microsoft `SendInput` 用于合成键盘、鼠标和按钮输入：https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
- Microsoft `LowLevelMouseProc` 用于低级鼠标 Hook 回调：https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc
- Microsoft `SetWindowsHookEx` 用于安装 Hook，例如 `WH_MOUSE_LL`：https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexa
- Microsoft `GetSystemMetrics` 用于读取屏幕、虚拟屏幕等系统指标：https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsystemmetrics

---

# 1. 产品需求文档 PRD

## 1.1 产品定位

本产品是一款 macOS 和 Windows 双平台桌面工具。用户在两台处于同一局域网的电脑上分别安装客户端后，可以把鼠标从一台电脑移动到另一台电脑，体验类似一套鼠标控制多台电脑。

产品目标类似 Synergy、Barrier、Mouse Without Borders，但 V1 只聚焦鼠标跨设备控制。

## 1.2 目标用户

- 同时使用 Mac 和 Windows PC 的办公用户。
- 开发者、设计师、测试、运维、数据分析师等多设备工作人群。
- 想减少物理鼠标/键盘切换或不想购买硬件 KVM 的用户。

## 1.3 用户价值

- 降低双电脑办公时的上下文切换成本。
- 让两台电脑在物理桌面上形成类似双显示器的操作体验。
- 不依赖账号和公网服务，局域网内即可使用。

## 1.4 V1 MVP 目标

- 两台设备能在同一局域网自动发现。
- 用户能通过配对码或 IP 直连建立信任。
- 用户能配置左右或上下布局。
- 鼠标到达配置边缘后能切换到另一台设备。
- 鼠标移动、点击、释放、滚轮能被远端设备执行。
- 断线后能自动重连并恢复连接状态。
- macOS 权限缺失时能清楚引导授权。

## 1.5 V1 非目标

- 不做账号登录。
- 不做云端设备列表。
- 不做公网穿透。
- 不做 Relay Server。
- 不做文件传输。
- 不做剪贴板同步。
- 不做键盘同步。
- 不做三台及以上设备布局。
- 不做移动端。

## 1.6 平台支持

V1 支持：

- Intel Mac。
- Apple Silicon Mac。
- Windows 10。
- Windows 11。

建议 QA 基线：

- macOS 12 Monterey 及以上。
- Windows 10 22H2 及以上。
- Windows 11 当前稳定版本。

## 1.7 核心用户流程

### 1.7.1 首次启动

用户打开应用后看到：

- 本机设备名称。
- 本机系统类型。
- 当前网络状态。
- macOS 权限状态或 Windows 输入能力状态。
- 已发现设备列表。

验收标准：

- 启动到主界面 `< 2s`。
- 权限缺失不会崩溃。
- macOS 缺失权限时显示明确引导按钮。

### 1.7.2 自动发现设备

用户在两台电脑启动客户端后，能在设备列表看到对方。

验收标准：

- 同 LAN 下对方设备在 `3s` 内出现。
- 显示设备名称、系统类型、IP 地址、应用版本、协议版本。
- 超过 `10s` 未收到发现广播后标记为离线或移除。
- 不显示本机自己。

### 1.7.3 配对设备

用户点击发现设备后，可以选择配对。

验收标准：

- 两端都弹出配对确认。
- 两端显示同一个 6 位配对码。
- 任一端拒绝则配对失败。
- 配对码 `120s` 后过期。
- 配对成功后保存远端设备 ID、公钥、名称、系统类型、最近地址。

### 1.7.4 IP 直连

当 mDNS 或 UDP Broadcast 不可用时，用户可以输入 IP 和端口连接。

验收标准：

- 支持 `192.168.x.x` 或 `hostname.local`。
- IP 直连仍必须走配对确认。
- 连接成功后保存最近地址。

### 1.7.5 屏幕布局配置

用户能设置远端设备相对本机的位置：

- 左。
- 右。
- 上。
- 下。

验收标准：

- 布局保存后重启仍生效。
- UI 显示两个屏幕块，本机和远端设备名称清晰可见。
- 当前可触发切换的边缘有高亮提示。
- 只有配对设备可保存布局。

### 1.7.6 鼠标无缝切换

用户把鼠标移动到配置边缘后，控制权切到远端。

验收标准：

- 左右布局：触达左/右边缘触发。
- 上下布局：触达上/下边缘触发。
- 切换后远端能执行移动、点击、释放、滚轮。
- 从远端对应反向边缘返回本机。
- 状态栏显示当前控制设备。
- 有紧急断开/停止控制入口。

### 1.7.7 状态同步和断线重连

应用维护当前连接状态和控制状态。

验收标准：

- 心跳 `1000ms` 一次。
- `3000ms` 内发现连接失效。
- 断线进入 `Reconnecting`。
- 按指数退避自动重连。
- 重连成功后回到 `ConnectedIdle`，不自动继续远程控制，避免误操作。

## 1.8 功能需求

### 1.8.1 设备发现

必需能力：

- 局域网自动发现。
- 显示设备名称。
- 显示系统类型。
- 显示 IP 地址。
- 一键连接。

推荐方案：

- 主路径：mDNS/Zeroconf。
- 兜底路径：UDP Broadcast。
- 手动路径：IP 直连。

mDNS 服务名：

```text
_mac22win._tcp.local
```

TXT 记录：

```text
device_id=<uuid>
device_name=<name>
os=macos|windows
arch=x86_64|aarch64
app_version=0.1.0
protocol_version=1
pairing=t|f
```

### 1.8.2 设备配对

配对方式：

- 从发现列表发起配对。
- 输入 IP 发起配对。

防误连接设计：

- 双端确认。
- 6 位短码。
- 过期时间。
- 展示远端设备名称、系统类型、IP。
- 已配对设备公钥不匹配时拒绝连接。

### 1.8.3 屏幕布局管理

布局选项：

- 远端在左。
- 远端在右。
- 远端在上。
- 远端在下。

边缘触发规则：

```text
Left:   x <= virtual_screen_min_x
Right:  x >= virtual_screen_max_x - 1
Top:    y <= virtual_screen_min_y
Bottom: y >= virtual_screen_max_y - 1
```

防误触建议：

- 默认 `corner_guard_px = 32`，屏幕角落附近不触发切换。
- 如测试中误触较多，V1.1 可增加 `edge_dwell_ms = 150` 的停留阈值。

UI 方案：

- 左侧：设备列表和连接状态。
- 右侧：布局编辑器。
- 布局编辑器使用两个屏幕矩形。
- 方向使用分段控件：左、右、上、下。
- 当前触发边缘以高亮线展示。
- 底部状态栏显示：`Ready`、`Missing Permission`、`Disconnected`、`Controlling Remote`、`Controlled By Remote`。

### 1.8.4 鼠标控制

主控端监听：

- `MouseMove`。
- `MouseDown`。
- `MouseUp`。
- `Wheel`。

主控端处理：

- 读取当前屏幕虚拟边界。
- 检测是否触达布局对应边缘。
- 触发 `ControlEnter`。
- 远控模式下将鼠标事件转换为协议帧。

被控端接收：

- `MouseMove`。
- `MouseDown`。
- `MouseUp`。
- `Wheel`。

被控端处理：

- 校验 session。
- 将协议事件转换为平台输入事件。
- 调用系统 API 注入鼠标。
- 更新远端鼠标位置状态。

### 1.8.5 实时通信

V1 需求：

- LAN 优先。
- 延迟尽可能低。
- 鼠标点击事件不能乱序或丢失。
- 支持心跳和自动恢复。

推荐：

- TCP 长连接。
- `TCP_NODELAY`。
- 二进制长度前缀协议。
- 鼠标移动可合并，按钮和滚轮不可随意丢弃。

### 1.8.6 状态同步

同步内容：

- 当前控制设备。
- 鼠标位置摘要。
- 连接状态。
- 心跳 RTT。
- 重连次数。

状态枚举：

```rust
pub enum SessionState {
    Disconnected,
    Discovered,
    Pairing,
    Paired,
    Connecting,
    ConnectedIdle,
    ControllingRemote,
    ControlledByRemote,
    Reconnecting,
    Error(SessionError),
}
```

## 1.9 非功能需求

性能：

- 启动时间 `< 2s`。
- 鼠标切换延迟 `< 20ms`，口径为本机边缘检测到远端首次成功注入。
- 空闲 CPU 尽量 `< 2%`。
- 活跃远控 CPU 尽量 `< 8%`。
- 稳态内存 `< 100MB`，需注意系统 WebView 实际占用存在平台差异。

可靠性：

- 权限缺失不崩溃。
- 网络断开不崩溃。
- 远端退出不崩溃。
- 30 分钟持续控制无明显内存增长。

安全：

- 未配对设备不能控制本机。
- 配对必须双端确认。
- 已信任设备的公钥变化必须拒绝并提示。
- V1 只承诺局域网内基础安全，不宣传企业级安全。

---

# 2. 技术架构设计文档

## 2.1 技术选型对比

| 方案 | 优点 | 缺点 | 结论 |
| --- | --- | --- | --- |
| Electron + Node.js | UI 生态成熟，开发快，跨平台经验多 | 内存高，内置 Chromium，系统输入能力依赖 native addon 或 sidecar，`<100MB` 压力大 | 不推荐 V1 |
| Tauri + Rust | 轻量，性能好，Rust 适合系统 API、网络和状态机，后续可抽 daemon | Rust 学习成本更高，部分平台 API 需要 unsafe/FFI | 推荐 |
| Wails + Go | Go 开发效率高，系统 WebView，内存低于 Electron | Go 在 macOS Event Tap、Windows Hook/Input 这类桌面底层生态不如 Rust 顺手 | 可作为备选 |

最终选择：

```text
Tauri + Rust + TypeScript
```

## 2.2 总体架构

```text
+-----------------------------------+
| Tauri Frontend                    |
| 设备列表 / 配对 / 布局 / 状态 UI     |
+----------------+------------------+
                 |
                 | Tauri Commands / Events
                 |
+----------------v------------------+
| Rust Core                         |
| 状态机 / 配置 / 配对 / 协议 / 日志    |
+----+--------------+------------+--+
     |              |            |
+----v----+   +-----v-----+ +----v----------------+
| Discovery | | Transport | | Platform Input      |
| mDNS/UDP  | | TCP       | | macOS / Windows     |
+----------+ +-----------+ +---------------------+
```

运行模型：

- 两端运行同一个客户端。
- 每个客户端同时具备主控端和被控端能力。
- V1 同一时间只允许一个活跃 peer。
- 高频鼠标事件只在 Rust 内部流转，不进入前端。

## 2.3 进程模型

V1：

- 单 Tauri 应用进程。
- Rust Core 内部启动网络、发现、输入监听线程。
- 平台 Hook 独立线程，避免阻塞 UI 和 async runtime。

后续：

- Rust Core 可抽为后台 daemon/service。
- Tauri UI 通过本地 IPC 管理 daemon。
- macOS 使用 LaunchAgent，Windows 使用用户级后台进程；需要管理员控制能力时再考虑辅助服务。

## 2.4 核心模块

| 模块 | 职责 |
| --- | --- |
| `app` | 生命周期、依赖装配、后台任务启动/停止 |
| `identity` | 本机设备 ID、公私钥、设备名称、系统信息 |
| `storage` | 配置、配对设备、布局、本地身份的持久化 |
| `discovery` | mDNS 发布/浏览，UDP Broadcast 兜底，发现缓存 |
| `pairing` | 配对码、双端确认、信任保存 |
| `network` | TCP 监听/连接、帧解析、心跳、重连 |
| `protocol` | wire message、版本协商、序列号、错误码 |
| `session` | 会话状态机、控制权切换、恢复策略 |
| `input` | 鼠标监听、边缘检测、输入注入 |
| `ui_api` | Tauri command/event，与前端交互 |
| `telemetry` | 日志、指标、诊断信息 |

## 2.5 macOS 实现方案

### 2.5.1 鼠标监听

推荐 API：

- `CGEventTapCreate`。
- `CFMachPortCreateRunLoopSource`。
- `CFRunLoopAddSource`。

推荐事件：

- `kCGEventMouseMoved`。
- `kCGEventLeftMouseDown`。
- `kCGEventLeftMouseUp`。
- `kCGEventRightMouseDown`。
- `kCGEventRightMouseUp`。
- `kCGEventOtherMouseDown`。
- `kCGEventOtherMouseUp`。
- `kCGEventScrollWheel`。

实现建议：

- 优先使用 `kCGSessionEventTap`，避免非 root 下 HID 入口 Tap 权限问题。
- 监听回调必须极短，只做事件转换和 channel 发送。
- 不在回调里做网络 IO、日志大对象序列化或 UI 通知。
- 事件 Tap 被系统禁用时尝试重新启用。

### 2.5.2 鼠标注入

推荐 API：

- `CGEventCreateMouseEvent`。
- `CGEventSetIntegerValueField`。
- `CGEventPost`。
- 需要时使用 `CGWarpMouseCursorPosition` 做光标定位。

实现内容：

- `MouseMove` 转为移动或绝对定位。
- `MouseDown`/`MouseUp` 映射左键、右键、中键和其他按钮。
- `Wheel` 映射滚轮事件。

权限：

- 注入鼠标通常需要 `Accessibility`。

### 2.5.3 macOS 权限

`Accessibility`：

- 用途：模拟系统鼠标输入，控制其他 app。
- 引导：打开 System Settings > Privacy & Security > Accessibility。
- 检测：可通过 `AXIsProcessTrustedWithOptions` 或相关辅助功能检测。

`Input Monitoring`：

- 用途：监听全局输入事件。
- 检测：`CGPreflightListenEventAccess`。
- 请求：`CGRequestListenEventAccess`，具体行为取决于系统版本。
- 引导：打开 System Settings > Privacy & Security > Input Monitoring。

`Screen Recording`：

- V1 不需要。
- 不应用于鼠标控制、点击或滚轮注入。
- 未来做屏幕预览、截图辅助布局、远程画面时再申请。

### 2.5.4 推荐 Rust crate

- `core-graphics`：CoreGraphics 类型和 CGEvent。
- `core-foundation`：RunLoop、CFMachPort 等。
- `objc2`：必要 Objective-C 桥接。
- `accessibility-sys` 或直接 FFI：辅助功能权限检测。

可用于 spike 但不建议作为最终核心抽象：

- `rdev`：跨平台监听/模拟更快验证，但底层控制和权限细节不够透明。

## 2.6 Windows 实现方案

### 2.6.1 鼠标监听

推荐 API：

- `SetWindowsHookExW`。
- `WH_MOUSE_LL`。
- `LowLevelMouseProc`。
- `CallNextHookEx`。

实现建议：

- 单独创建 Hook 线程。
- 线程内维护 Windows message loop。
- Hook 回调只转换事件并投递到 Rust channel。
- 回调必须快速返回，避免系统移除 Hook 或造成输入卡顿。

### 2.6.2 鼠标注入

推荐 API：

- `SendInput`。
- `INPUT`。
- `MOUSEINPUT`。

事件映射：

- 移动：`MOUSEEVENTF_MOVE`。
- 绝对定位：`MOUSEEVENTF_ABSOLUTE`，多屏时配合 `MOUSEEVENTF_VIRTUALDESK`。
- 左键：`MOUSEEVENTF_LEFTDOWN` / `MOUSEEVENTF_LEFTUP`。
- 右键：`MOUSEEVENTF_RIGHTDOWN` / `MOUSEEVENTF_RIGHTUP`。
- 中键：`MOUSEEVENTF_MIDDLEDOWN` / `MOUSEEVENTF_MIDDLEUP`。
- 滚轮：`MOUSEEVENTF_WHEEL` / `MOUSEEVENTF_HWHEEL`。

屏幕信息：

- `GetSystemMetrics(SM_XVIRTUALSCREEN)`。
- `GetSystemMetrics(SM_YVIRTUALSCREEN)`。
- `GetSystemMetrics(SM_CXVIRTUALSCREEN)`。
- `GetSystemMetrics(SM_CYVIRTUALSCREEN)`。
- 后续可补 `EnumDisplayMonitors` 和 DPI API。

权限：

- 普通用户权限足够完成正常桌面应用的监听和注入。
- 不要求管理员权限。
- 非管理员进程不能可靠控制管理员权限窗口、UAC 安全桌面或受保护程序。

### 2.6.3 推荐 Rust crate

- `windows`：Win32 API 绑定。
- `tokio`：网络和异步任务。
- `tracing`：日志。

## 2.7 输入事件热路径

本机正常模式：

```text
OS Mouse Event -> Platform Hook -> Edge Detector -> 不拦截本机默认行为
```

触发切换：

```text
Edge Detector -> Session(ControlEnter) -> Network -> Remote Inject Initial Position
```

远控模式：

```text
OS Mouse Event -> Platform Hook -> Delta Normalize -> Network Frame -> Remote Inject
```

返回本机：

```text
Remote Edge Detector -> ControlLeave -> Original Device ConnectedIdle
```

关键约束：

- 鼠标事件不要穿过前端。
- 移动事件可以合并。
- 点击、释放、滚轮要保序。
- 队列必须有上限，避免网络异常时内存增长。

## 2.8 光标处理策略

V1 推荐策略：

- 切换到远端后，本机光标停在触发边缘内侧 1 到 2 像素。
- 远端光标出现在对应反向边缘。
- 远控期间使用鼠标 delta 驱动远端。
- 不强行承诺完全隐藏本机光标，隐藏/抑制可作为 V1.1 优化。

原因：

- macOS 和 Windows 对全局事件拦截、抑制和自注入过滤的行为差异较大。
- V1 先保证可用、低延迟和不乱序。

---

# 3. 模块拆分文档

## 3.1 Rust Core 模块

```text
src-tauri/src/
  main.rs
  lib.rs
  app/
  config/
  discovery/
  identity/
  input/
  network/
  pairing/
  protocol/
  session/
  storage/
  telemetry/
  ui_api/
```

## 3.2 `app`

职责：

- 初始化配置、身份、日志。
- 启动 discovery、network、input。
- 管理 app shutdown。
- 向 UI 广播状态。

关键类型：

```rust
pub struct AppContext {
    pub identity: DeviceIdentity,
    pub config: Arc<RwLock<AppConfig>>,
    pub discovery: DiscoveryService,
    pub pairing: PairingService,
    pub network: NetworkManager,
    pub session: SessionController,
    pub input: PlatformInputManager,
}
```

## 3.3 `identity`

职责：

- 首次启动生成 `device_id`。
- 读取系统 hostname 作为默认设备名。
- 生成本机密钥对。
- 提供公开身份给 discovery/pairing。

验收：

- 重启后 `device_id` 不变。
- 清空身份文件后重新生成。

## 3.4 `discovery`

职责：

- 发布 `_mac22win._tcp.local`。
- 浏览同服务类型设备。
- 维护发现缓存。
- UDP Broadcast 兜底。
- 过滤本机。

验收：

- 设备 3 秒内出现。
- 10 秒未见标记 stale。
- mDNS 不可用时可以通过 IP 直连继续使用。

## 3.5 `pairing`

职责：

- 创建配对请求。
- 生成 6 位配对码。
- 管理过期时间。
- 双端确认。
- 写入 trusted peers。

状态：

```text
Idle -> Requesting -> WaitingRemoteConfirm -> WaitingLocalConfirm -> Paired
Idle -> IncomingRequest -> WaitingLocalConfirm -> WaitingRemoteConfirm -> Paired
Any -> Rejected | Expired | Failed
```

## 3.6 `network`

职责：

- TCP listener。
- TCP connector。
- frame encode/decode。
- heartbeat。
- reconnect。
- backpressure。

关键要求：

- 开启 `TCP_NODELAY`。
- 帧最大 `64 KiB`。
- 心跳超时 `3000ms`。
- 重连指数退避。

## 3.7 `protocol`

职责：

- 定义 wire message。
- 协议版本协商。
- 序列号。
- 错误码。
- parser 安全。

验收：

- 截断帧不 panic。
- 超大帧拒绝。
- 未知协议版本拒绝。
- move coalescing 不跨越 button event。

## 3.8 `session`

职责：

- 管理连接状态。
- 管理控制权。
- 响应边缘触发。
- 控制 input capture/inject。
- 处理断线和恢复。

状态机：

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

## 3.9 `input`

职责：

- 平台无关输入接口。
- macOS 实现。
- Windows 实现。
- 屏幕边界读取。
- 边缘检测。
- 自注入事件过滤。

接口草案：

```rust
pub trait InputPlatform {
    fn permissions(&self) -> PermissionStatus;
    fn request_permissions(&self) -> Result<()>;
    fn screen_topology(&self) -> Result<ScreenTopology>;
    fn start_capture(&self, tx: InputEventSender) -> Result<InputCaptureHandle>;
    fn inject(&self, event: RemoteMouseEvent) -> Result<()>;
}
```

## 3.10 `storage`

职责：

- 配置文件读写。
- trusted peers 读写。
- identity 读写。
- 原子写。
- schema migration。

## 3.11 `ui_api`

Tauri Commands：

```rust
#[tauri::command]
async fn get_app_status() -> UiAppStatus;

#[tauri::command]
async fn list_discovered_devices() -> Vec<UiDiscoveredDevice>;

#[tauri::command]
async fn start_pairing(device_id: String) -> Result<UiPairingSession, UiError>;

#[tauri::command]
async fn confirm_pairing(pairing_id: String) -> Result<(), UiError>;

#[tauri::command]
async fn connect_peer(peer_id: String) -> Result<(), UiError>;

#[tauri::command]
async fn connect_by_ip(address: String) -> Result<(), UiError>;

#[tauri::command]
async fn save_layout(layout: UiLayoutConfig) -> Result<(), UiError>;

#[tauri::command]
async fn disconnect() -> Result<(), UiError>;

#[tauri::command]
async fn open_permission_settings(permission: PermissionKind) -> Result<(), UiError>;
```

Frontend events：

```text
device:discovered
device:stale
pairing:request
pairing:updated
session:state
permission:updated
network:metrics
```

---

# 4. 数据结构设计

## 4.1 基础类型

```rust
pub type DeviceId = String;
pub type PeerId = String;
pub type PairingId = String;
pub type SessionId = String;
pub type TimestampMs = u64;
```

## 4.2 枚举

```rust
pub enum OsType {
    Macos,
    Windows,
    Unknown,
}

pub enum CpuArch {
    X86_64,
    Aarch64,
    Unknown,
}

pub enum LayoutDirection {
    Left,
    Right,
    Top,
    Bottom,
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u8),
}

pub enum PermissionKind {
    Accessibility,
    InputMonitoring,
    ScreenRecording,
    WindowsInput,
}

pub enum PermissionState {
    Granted,
    Denied,
    NotDetermined,
    Unsupported,
    Unknown,
}
```

## 4.3 本机身份

```rust
pub struct DeviceIdentity {
    pub schema_version: u16,
    pub device_id: DeviceId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub app_version: String,
    pub protocol_version: u16,
    pub public_key: Vec<u8>,
    pub private_key_ref: PrivateKeyRef,
    pub created_at_ms: TimestampMs,
}

pub enum PrivateKeyRef {
    FileEncrypted { path: String },
    Keychain { service: String, account: String },
    WindowsCredential { target: String },
}
```

## 4.4 可信设备

```rust
pub struct TrustedPeer {
    pub peer_id: PeerId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub public_key: Vec<u8>,
    pub last_known_addresses: Vec<String>,
    pub last_seen_ms: Option<TimestampMs>,
    pub paired_at_ms: TimestampMs,
    pub app_version_at_pairing: String,
    pub protocol_version: u16,
    pub trust_state: TrustState,
}

pub enum TrustState {
    Trusted,
    Blocked,
    Removed,
}
```

## 4.5 发现设备

```rust
pub struct DiscoveredPeer {
    pub device_id: DeviceId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub app_version: String,
    pub protocol_version: u16,
    pub addresses: Vec<String>,
    pub service_port: u16,
    pub pairing_available: bool,
    pub last_seen_ms: TimestampMs,
    pub source: DiscoverySource,
}

pub enum DiscoverySource {
    Mdns,
    UdpBroadcast,
    ManualIp,
}
```

## 4.6 布局配置

```rust
pub struct LayoutConfig {
    pub peer_id: PeerId,
    pub direction: LayoutDirection,
    pub edge_thickness_px: u32,
    pub corner_guard_px: u32,
    pub enabled: bool,
    pub updated_at_ms: TimestampMs,
}
```

默认值：

```json
{
  "direction": "right",
  "edge_thickness_px": 1,
  "corner_guard_px": 32,
  "enabled": true
}
```

## 4.7 屏幕拓扑

```rust
pub struct ScreenTopology {
    pub virtual_bounds: Rect,
    pub primary_display_id: String,
    pub displays: Vec<DisplayInfo>,
    pub scale_factor: f64,
    pub captured_at_ms: TimestampMs,
}

pub struct DisplayInfo {
    pub display_id: String,
    pub name: Option<String>,
    pub bounds: Rect,
    pub work_area: Rect,
    pub scale_factor: f64,
    pub is_primary: bool,
}

pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

坐标策略：

- 内部统一使用逻辑像素。
- 平台边界层负责转换 Retina/Windows DPI。
- Windows 绝对注入需要转换为 `0..65535` 范围。

## 4.8 鼠标事件

```rust
pub enum LocalMouseEvent {
    Move(MouseMove),
    Down(MouseButtonEvent),
    Up(MouseButtonEvent),
    Wheel(MouseWheelEvent),
}

pub struct MouseMove {
    pub position: Point,
    pub delta: Delta,
    pub timestamp_ms: TimestampMs,
}

pub struct Delta {
    pub dx: f64,
    pub dy: f64,
}

pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub position: Point,
    pub timestamp_ms: TimestampMs,
}

pub struct MouseWheelEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub position: Point,
    pub timestamp_ms: TimestampMs,
}
```

远端事件：

```rust
pub enum RemoteMouseEvent {
    MoveDelta {
        dx: f32,
        dy: f32,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
    MoveAbsolute {
        x: f32,
        y: f32,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
    Down {
        button: MouseButton,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
    Up {
        button: MouseButton,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
    Wheel {
        dx: f32,
        dy: f32,
        seq: u32,
        timestamp_ms: TimestampMs,
    },
}
```

## 4.9 会话快照

```rust
pub struct SessionSnapshot {
    pub session_id: Option<SessionId>,
    pub peer_id: Option<PeerId>,
    pub state: SessionState,
    pub control_owner: ControlOwner,
    pub local_pointer: Option<Point>,
    pub remote_pointer: Option<Point>,
    pub last_heartbeat_rtt_ms: Option<u32>,
    pub connected_since_ms: Option<TimestampMs>,
    pub updated_at_ms: TimestampMs,
}

pub enum ControlOwner {
    Local,
    Remote,
    None,
}
```

## 4.10 应用配置

```rust
pub struct AppConfig {
    pub schema_version: u16,
    pub local_device_name_override: Option<String>,
    pub network: NetworkConfig,
    pub discovery: DiscoveryConfig,
    pub layouts: Vec<LayoutConfig>,
    pub trusted_peers: Vec<TrustedPeer>,
    pub ui: UiConfig,
    pub updated_at_ms: TimestampMs,
}

pub struct NetworkConfig {
    pub listen_port: u16,
    pub connect_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub heartbeat_timeout_ms: u64,
    pub reconnect_min_delay_ms: u64,
    pub reconnect_max_delay_ms: u64,
}

pub struct DiscoveryConfig {
    pub mdns_enabled: bool,
    pub udp_broadcast_enabled: bool,
    pub udp_port: u16,
    pub announce_interval_ms: u64,
    pub stale_after_ms: u64,
}

pub struct UiConfig {
    pub start_minimized: bool,
    pub show_diagnostics: bool,
    pub last_selected_peer_id: Option<PeerId>,
}
```

---

# 5. 网络协议设计

## 5.1 传输协议比较

| 协议 | 延迟 | 可靠性 | 实现复杂度 | V1 结论 |
| --- | --- | --- | --- | --- |
| TCP | LAN 下低，配合 `TCP_NODELAY` 足够快 | 有序可靠 | 低 | 推荐 |
| WebSocket | 低，但有额外握手和 framing | 有序可靠 | 中 | 不需要浏览器兼容，暂不选 |
| QUIC | 很低，支持多流和 datagram | 好 | 高 | V2 候选 |
| UDP + 自定义协议 | 理论最低 | 需自建排序、重传、拥塞、丢包策略 | 高 | V1 不推荐 |

V1 最终方案：

```text
TCP + TCP_NODELAY + 长度前缀二进制帧
```

## 5.2 端口

```text
TCP Control: 42424
UDP Discovery Fallback: 42425
mDNS Service: _mac22win._tcp.local
```

## 5.3 帧格式

```text
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+---------------------------------------------------------------+
|                        length u32 BE                          |
+---------------+---------------+-------------------------------+
| version u16 BE | type u16 BE   | flags u16 BE                  |
+---------------+---------------+-------------------------------+
| seq u32 BE                                                     |
+---------------------------------------------------------------+
| payload ...                                                    |
+---------------------------------------------------------------+
```

规则：

- `length` 表示后续 header + payload 字节数。
- V1 `version = 1`。
- V1 最大帧 `64 KiB`。
- payload 推荐用 `bincode` 或 `postcard`。
- 高频鼠标事件不使用 JSON。

## 5.4 消息类型

```rust
pub enum MessageType {
    Hello = 1,
    HelloAck = 2,
    PairingRequest = 10,
    PairingResponse = 11,
    PairingConfirm = 12,
    PairingReject = 13,
    SessionStart = 20,
    SessionState = 21,
    ControlEnter = 30,
    ControlLeave = 31,
    MouseMove = 40,
    MouseButton = 41,
    MouseWheel = 42,
    HeartbeatPing = 50,
    HeartbeatPong = 51,
    Error = 90,
    Goodbye = 99,
}
```

## 5.5 Hello

```rust
pub struct Hello {
    pub device_id: DeviceId,
    pub device_name: String,
    pub os: OsType,
    pub arch: CpuArch,
    pub app_version: String,
    pub protocol_version: u16,
    pub public_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub supported_features: Vec<String>,
}
```

规则：

- 协议版本不兼容则返回 `UnsupportedProtocol`。
- 非 pairing 模式下未知 peer 直接拒绝。
- 已配对 peer 的 public key 不匹配则拒绝并提示身份异常。

## 5.6 配对协议

配对请求：

```rust
pub struct PairingRequest {
    pub pairing_id: PairingId,
    pub from: DeviceIdentityPublic,
    pub nonce: Vec<u8>,
    pub requested_at_ms: TimestampMs,
    pub expires_at_ms: TimestampMs,
}
```

配对响应：

```rust
pub struct PairingResponse {
    pub pairing_id: PairingId,
    pub from: DeviceIdentityPublic,
    pub nonce: Vec<u8>,
    pub code_fingerprint: String,
    pub accepted_for_confirmation: bool,
}
```

配对确认：

```rust
pub struct PairingConfirm {
    pub pairing_id: PairingId,
    pub confirmed: bool,
    pub confirmed_at_ms: TimestampMs,
}
```

配对码生成：

```text
code = first_6_decimal_digits(
  SHA256(
    min(device_id_a, device_id_b) ||
    max(device_id_a, device_id_b) ||
    nonce_a ||
    nonce_b ||
    public_key_a ||
    public_key_b
  )
)
```

## 5.7 控制消息

```rust
pub struct ControlEnter {
    pub session_id: SessionId,
    pub entry_edge: Edge,
    pub initial_position: PointWire,
    pub sent_at_ms: TimestampMs,
}

pub struct ControlLeave {
    pub session_id: SessionId,
    pub reason: ControlLeaveReason,
    pub exit_edge: Option<Edge>,
    pub sent_at_ms: TimestampMs,
}
```

## 5.8 鼠标消息

移动：

```rust
pub struct MouseMoveMsg {
    pub session_id: SessionId,
    pub dx: f32,
    pub dy: f32,
    pub absolute_x: Option<f32>,
    pub absolute_y: Option<f32>,
    pub sent_at_ms: TimestampMs,
}
```

按钮：

```rust
pub struct MouseButtonMsg {
    pub session_id: SessionId,
    pub button: MouseButton,
    pub action: ButtonAction,
    pub sent_at_ms: TimestampMs,
}

pub enum ButtonAction {
    Down,
    Up,
}
```

滚轮：

```rust
pub struct MouseWheelMsg {
    pub session_id: SessionId,
    pub dx: f32,
    pub dy: f32,
    pub sent_at_ms: TimestampMs,
}
```

事件优先级：

- `MouseButton`：高优先级，不丢弃。
- `MouseWheel`：高优先级，谨慎合并。
- `MouseMove`：低优先级，可合并。
- `ControlEnter/Leave`：最高优先级。

## 5.9 心跳和重连

心跳：

```rust
pub struct HeartbeatPing {
    pub session_id: Option<SessionId>,
    pub ping_id: u32,
    pub sent_at_ms: TimestampMs,
}

pub struct HeartbeatPong {
    pub session_id: Option<SessionId>,
    pub ping_id: u32,
    pub sent_at_ms: TimestampMs,
    pub received_ping_at_ms: TimestampMs,
}
```

默认：

- `heartbeat_interval_ms = 1000`。
- `heartbeat_timeout_ms = 3000`。

重连：

```text
delay = min(max_delay, min_delay * 2^attempt) + jitter(0..250ms)
```

默认：

- `min_delay = 500ms`。
- `max_delay = 10000ms`。
- 稳定连接 `30s` 后重置 attempt。

---

# 6. 本地存储设计

## 6.1 存储位置

使用 Tauri app config/data 目录。

逻辑结构：

```text
<app_config_dir>/
  config.json
  identity.json
  trusted_peers.json
  layouts.json
  logs/
    app.log
```

平台示例：

- macOS：`~/Library/Application Support/com.mac22win.app/`
- Windows：`%APPDATA%\com.mac22win.app\`

## 6.2 `identity.json`

保存本机稳定身份：

```json
{
  "schema_version": 1,
  "device_id": "7fd37b26-caf6-43db-b6c1-16d9b48fd2d2",
  "device_name": "Alice MacBook Pro",
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

## 6.3 `trusted_peers.json`

保存配对设备：

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

## 6.4 `layouts.json`

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

## 6.5 `config.json`

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

## 6.6 原子写策略

写入流程：

```text
serialize -> write <file>.tmp -> flush -> rename tmp to real file
```

启动恢复：

- 主文件正常则读取主文件。
- 主文件损坏但 tmp 正常则恢复 tmp。
- 都损坏则备份为 `*.corrupt.<timestamp>`，使用默认配置启动。

## 6.7 密钥存储

推荐：

- macOS 使用 Keychain。
- Windows 使用 Credential Manager 或 DPAPI。

MVP 可接受兜底：

- 本地文件保存私钥。
- 文件权限限制为当前用户。
- 抽象 `SecretStore`，后续可迁移到系统安全存储。

---

# 7. 项目目录结构设计

推荐项目结构：

```text
mac22win/
  README.md
  package.json
  pnpm-lock.yaml
  index.html
  vite.config.ts
  tsconfig.json
  docs/
    00-master-plan-zh-CN.md
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
    src/
      main.rs
      lib.rs
      app/
      config/
      discovery/
      identity/
      input/
        mod.rs
        types.rs
        edge.rs
        macos.rs
        windows.rs
        noop.rs
      network/
      pairing/
      protocol/
      session/
      storage/
      telemetry/
      ui_api/
      platform/
    tests/
      protocol_tests.rs
      storage_tests.rs
      pairing_tests.rs
```

推荐依赖：

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
bincode = "1"
sha2 = "0.10"
rand = "0.8"
mdns-sd = "0.13"
```

Windows：

```toml
windows = { version = "0.58", features = [
  "Win32_UI_Input_KeyboardAndMouse",
  "Win32_UI_WindowsAndMessaging",
  "Win32_Foundation",
  "Win32_Graphics_Gdi"
] }
```

macOS：

```toml
core-graphics = "0.24"
core-foundation = "0.10"
objc2 = "0.5"
```

实现时应重新确认依赖最新版本。

---

# 8. MVP 开发路线图

## Phase 0：平台能力 Spike

目标：

- 先证明 macOS 和 Windows 都能监听和注入鼠标。

任务：

- 最小 Tauri app。
- macOS 权限检测。
- macOS `CGEventTapCreate` 鼠标监听。
- macOS `CGEventPost` 鼠标注入。
- Windows `WH_MOUSE_LL` 鼠标监听。
- Windows `SendInput` 鼠标注入。
- 记录 capture 到 inject 的本机延迟。

退出标准：

- macOS 授权后能监听 move/down/up/wheel。
- macOS 能注入 move/down/up/wheel。
- Windows 普通用户能监听和注入。
- 已知权限限制写入风险文档。

## Phase 1：项目骨架

任务：

- 创建 Tauri + TypeScript 项目。
- 创建 Rust 模块树。
- 实现 identity/config/storage。
- 实现日志。
- UI 显示本机状态和权限状态。

退出标准：

- 双平台可启动。
- 本机身份持久化。
- 权限状态可见。

## Phase 2：设备发现

任务：

- mDNS publish。
- mDNS browse。
- discovered cache。
- stale 清理。
- UDP Broadcast 兜底。
- 设备列表 UI。

退出标准：

- 同 LAN 设备 3 秒内出现。
- 10 秒未见后 stale。
- 显示设备名、OS、IP、版本。

## Phase 3：配对

任务：

- PairingRequest/Response/Confirm。
- 6 位配对码。
- 双端确认 UI。
- trusted peers 持久化。
- IP 直连。

退出标准：

- 配对成功后重启仍可信。
- 拒绝/过期不会保存 trust。
- 公钥不匹配会拒绝。

## Phase 4：TCP 会话和协议

任务：

- frame parser。
- message encode/decode。
- TCP listener/client。
- `TCP_NODELAY`。
- Hello 校验。
- heartbeat。
- reconnect。
- 协议测试。

退出标准：

- 已配对设备可自动连接。
- 心跳 RTT 可见。
- 3 秒内发现断线。
- 自动重连成功。

## Phase 5：布局管理

任务：

- 布局存储。
- 布局编辑器 UI。
- 读取屏幕拓扑。
- 边缘检测。
- corner guard。

退出标准：

- 左/右/上/下都能触发检测。
- 布局重启后保留。

## Phase 6：鼠标无缝切换

任务：

- input capture 接入 session。
- 边缘触发 `ControlEnter`。
- 远端注入初始位置。
- 移动 delta 流式发送。
- 点击/释放/滚轮发送。
- 反向边缘返回。
- 紧急停止。
- 自注入事件过滤。

退出标准：

- macOS 控制 Windows。
- Windows 控制 macOS。
- 移动、点击、释放、滚轮可用。
- 四个方向布局可用。

## Phase 7：稳定性和打包

任务：

- macOS 新用户权限流程测试。
- Windows 普通用户测试。
- Wi-Fi 和有线 LAN 测试。
- 诊断面板。
- 日志轮转。
- 队列和 move coalescing 调优。
- macOS/Windows 安装包。

退出标准：

- 30 分钟连续控制不崩溃。
- 启动 `< 2s`。
- 正常 LAN 下切换 `< 20ms`。
- 内存目标达成或明确记录平台 WebView 实测差异。

---

# 9. 风险评估

## 9.1 高风险清单

| 风险 | 影响 | 概率 | 缓解 |
| --- | --- | --- | --- |
| macOS 权限导致监听/注入失败 | 核心功能不可用 | 高 | Phase 0 优先 Spike，权限 UI 清晰 |
| 本机光标抑制不稳定 | 体验粗糙 | 中 | V1 使用光标停靠 + delta 转发 |
| Windows 无法控制管理员窗口 | 用户误以为失败 | 中 | 文档和 UI 提示，V2 再考虑 elevated helper |
| mDNS 被网络阻断 | 发现失败 | 高 | UDP fallback + IP 直连 |
| Wi-Fi 延迟/抖动 | 鼠标不顺滑 | 中 | TCP_NODELAY、二进制协议、move 合并 |
| 自注入事件反馈循环 | 鼠标跳动 | 中 | 标记/过滤注入事件，远控期间忽略本机绝对位置 |
| DPI/Retina 坐标错误 | 光标落点不准 | 中 | 统一逻辑坐标，平台边界转换 |
| 未授权 LAN 设备尝试连接 | 安全风险 | 中 | 双端配对、公钥 pin、未知 peer 拒绝 |

## 9.2 macOS 特有风险

Accessibility：

- 注入系统输入依赖该权限。
- 用户授权后可能需要重启 app。
- dev build 和 packaged build 的权限身份可能不同。

Input Monitoring：

- 全局输入监听可能被阻止。
- 需要明确检测和引导。

Event Tap：

- 回调过慢可能被系统禁用。
- 必须保持短回调并提供恢复机制。

Screen Recording：

- V1 不应申请，避免不必要的隐私压力。

## 9.3 Windows 特有风险

UIPI/完整性级别：

- 普通权限 app 不能可靠注入到管理员权限 app。

Hook 线程：

- `WH_MOUSE_LL` 需要 message loop。
- 回调过慢影响系统输入。

安全软件：

- 全局 Hook 和输入注入可能被安全软件关注。
- 后续需要代码签名和清晰产品说明。

## 9.4 网络风险

Multicast 被禁：

- 企业/访客 Wi-Fi 可能禁用 mDNS。
- IP 直连必须做成一等入口。

防火墙：

- Windows 防火墙可能阻止入站 TCP。
- V1 提供排障提示，后续安装器可添加规则。

## 9.5 性能风险

内存：

- Tauri 更轻，但 WebView 内存取决于系统实现。
- 前端依赖要克制。

事件积压：

- 鼠标 move 高频可能堆积。
- 使用 bounded channel。
- move 可合并，button 不可丢。

---

# 10. V2/V3 演进方案

## 10.1 V2 建议

### 键盘共享

新增：

- KeyDown/KeyUp 捕获。
- 修饰键状态同步。
- 键盘布局映射。
- IME 兼容策略。

风险：

- 权限敏感度显著上升。
- 国际键盘和输入法复杂。

### 剪贴板文本同步

新增：

- 剪贴板 watcher。
- 文本同步。
- 去重和防循环。
- 用户开关。

风险：

- 剪贴板可能包含密码、Token。
- 建议默认关闭或首次明确提示。

### 多设备布局

新增：

- 从单 peer direction 升级为 layout graph。
- 支持三台及以上设备。
- 边缘冲突处理。

数据模型：

```rust
pub struct LayoutGraph {
    pub nodes: Vec<DeviceNode>,
    pub edges: Vec<LayoutEdge>,
}
```

### QUIC Transport

适用条件：

- TCP 实测无法满足延迟目标。
- 需要多个 stream。
- 需要 datagram 承载高频 pointer delta。

建议：

- 不在 V1 引入。
- V2 以 feature flag 做实验。

### 系统托盘体验

新增：

- 快速启用/禁用。
- 当前连接设备。
- 紧急断开。
- 最近错误状态。

## 10.2 V3 建议

### 账号和设备管理

新增：

- 用户账号。
- 设备注册。
- 信任撤销。
- 多设备同步配置。

前提：

- 本地 MVP 留存和稳定性已验证。
- 安全模型经过复盘。

### 公网穿透和 Relay

新增：

- Relay server。
- NAT traversal。
- QUIC 连接迁移。

风险：

- 延迟可能不适合鼠标控制。
- 运营成本和安全责任大幅增加。

### 文件传输

新增：

- 文件发送协议。
- 进度 UI。
- 冲突处理。
- 文件落点确认。

### 远程屏幕预览

新增：

- 屏幕采集。
- 编码。
- macOS Screen Recording 权限。
- 预览 UI。

建议：

- 与鼠标控制热路径隔离。

## 10.3 架构演进方向

V1：

```text
Tauri App Process = UI + Rust Core
```

V2/V3：

```text
Tauri UI <-> Local IPC <-> Rust Core Daemon <-> Network/Input
```

收益：

- UI 关闭后仍可运行。
- 权限和后台启动更清晰。
- 后续可做 helper/service 权限隔离。

## 10.4 推荐演进顺序

1. 稳定 LAN 鼠标共享。
2. 完善托盘和紧急停止。
3. 增加键盘共享。
4. 增加剪贴板文本同步。
5. 增加多设备布局。
6. 增加签名、更新和发布渠道。
7. 最后再考虑账号、公网和 Relay。

---

# 11. 给 AI Coding Agent 的实施指令摘要

建议按以下顺序实现：

1. 搭 Tauri 项目和 Rust 模块树。
2. 先做平台输入 Spike，不要先写完整 UI。
3. 实现 storage/identity/protocol 的单元测试。
4. 实现 mDNS/UDP discovery。
5. 实现 pairing 和 trusted peer。
6. 实现 TCP session、heartbeat、reconnect。
7. 实现 layout 和 edge detector。
8. 接入 mouse handoff。
9. 最后完善 UI、权限引导、诊断和打包。

硬性工程规则：

- 高频鼠标事件不得进入 WebView。
- Hook 回调不得做阻塞操作。
- TCP 必须开启 `TCP_NODELAY`。
- 所有队列必须 bounded。
- `MouseButton` 不得丢弃或乱序。
- parser 不得 `unwrap` 外部输入。
- macOS V1 不申请 Screen Recording。
- Windows V1 不要求管理员权限。


import type { LayoutDirection, PermissionState, SessionState, UiDevice } from "./types";

export type AppLanguage = "zh" | "en";

type DeviceStatus = UiDevice["status"];
type ControlOwner = "local" | "remote" | "none";

export interface AppCopy {
  languageName: string;
  loading: string;
  preferenceTitle: string;
  brand: string;
  brandTagline: string;
  tabs: {
    overview: string;
    devices: string;
    layout: string;
    permissions: string;
    network: string;
    about: string;
  };
  overview: {
    heroSubtitle: (sessionLabel: string) => string;
    refresh: string;
    emergencyStop: string;
    metricsTitle: string;
    metrics: {
      session: string;
      controlOwner: string;
      rtt: string;
      rttUnit: string;
      rttIdle: string;
      discovered: string;
      trusted: string;
    };
    quickActionsTitle: string;
    handoffToggle: string;
    handoffToggleHint: string;
    discoveryToggle: string;
    discoveryToggleHint: string;
    autoReconnectToggle: string;
    autoReconnectHint: string;
    liquidGlassToggle: string;
    liquidGlassHint: string;
  };
  devices: {
    title: string;
    discovered: (count: number) => string;
    localBadge: string;
    empty: string;
    emptyHint: string;
    pair: string;
    connect: string;
    forget: string;
    addManual: string;
  };
  permissions: {
    title: string;
    subtitle: string;
    accessibility: string;
    accessibilityHint: string;
    inputMonitoring: string;
    inputMonitoringHint: string;
    windowsInput: string;
    windowsInputHint: string;
    open: string;
  };
  network: {
    title: string;
    listenLabel: string;
    discoveryProto: string;
    heartbeat: string;
    heartbeatUnit: string;
    rttLabel: string;
    fallback: string;
    udpFallback: string;
    encryption: string;
    encryptionValue: string;
  };
  layout: {
    title: string;
    description: (peerName: string) => string;
    canvasDirectionLabel: (direction: string) => string;
    localTag: string;
    peerTag: string;
    dragHint: string;
    save: string;
    enabled: string;
    disabled: string;
    noPeerSelected: string;
    noPeerName: string;
    summary: {
      direction: string;
      cornerGuard: string;
      cornerGuardValue: (px: number) => string;
    };
  };
  about: {
    tagline: string;
    version: (version: string) => string;
    project: string;
    license: string;
    licenseValue: string;
  };
  statusStrip: {
    handoff: string;
    on: string;
    off: string;
    rtt: string;
    rttIdle: string;
    discovered: (count: number) => string;
  };
  languageToggleLabel: string;
  states: {
    layoutDirections: Record<LayoutDirection, string>;
    permissions: Record<PermissionState, string>;
    sessions: Record<SessionState, string>;
    controlOwners: Record<ControlOwner, string>;
    deviceStatuses: Record<DeviceStatus, string>;
  };
}

export const copy: Record<AppLanguage, AppCopy> = {
  zh: {
    languageName: "中文",
    loading: "正在加载 FlowLink",
    preferenceTitle: "偏好设置",
    brand: "FlowLink",
    brandTagline: "局域网鼠标无缝协同",
    tabs: {
      overview: "总览",
      devices: "设备",
      layout: "布局",
      permissions: "权限",
      network: "网络",
      about: "关于"
    },
    overview: {
      heroSubtitle: (sessionLabel) => `当前会话 · ${sessionLabel}`,
      refresh: "刷新",
      emergencyStop: "紧急停止",
      metricsTitle: "运行状态",
      metrics: {
        session: "会话",
        controlOwner: "控制权",
        rtt: "心跳延迟",
        rttUnit: "ms",
        rttIdle: "未连接",
        discovered: "发现设备",
        trusted: "可信设备"
      },
      quickActionsTitle: "快速设置",
      handoffToggle: "启用鼠标无缝切换",
      handoffToggleHint: "鼠标移动到布局边缘时自动接管远端",
      discoveryToggle: "局域网自动发现",
      discoveryToggleHint: "使用 mDNS / UDP 广播寻找同网段设备",
      autoReconnectToggle: "断线自动重连",
      autoReconnectHint: "网络恢复后自动续接，但不自动恢复远控",
      liquidGlassToggle: "液态玻璃外观",
      liquidGlassHint: "启用 macOS 原生 vibrancy / Windows Mica，让窗口透明融入桌面"
    },
    devices: {
      title: "设备列表",
      discovered: (count) => `已发现 ${count} 台设备`,
      localBadge: "本机",
      empty: "暂未发现设备",
      emptyHint: "确保两台设备在同一局域网，并已启用自动发现",
      pair: "配对",
      connect: "连接",
      forget: "取消信任",
      addManual: "手动添加"
    },
    permissions: {
      title: "权限",
      subtitle: "缺失权限会导致鼠标监听或注入失败",
      accessibility: "辅助功能",
      accessibilityHint: "用于向远端设备发送鼠标事件",
      inputMonitoring: "输入监听",
      inputMonitoringHint: "用于监听本机鼠标移动与点击",
      windowsInput: "Windows 输入能力",
      windowsInputHint: "普通用户权限即可，无需管理员",
      open: "前往设置"
    },
    network: {
      title: "网络",
      listenLabel: "监听端口",
      discoveryProto: "发现协议",
      heartbeat: "心跳间隔",
      heartbeatUnit: "ms",
      rttLabel: "当前 RTT",
      fallback: "兜底广播",
      udpFallback: "UDP 广播",
      encryption: "通道加密",
      encryptionValue: "Noise XX · ChaCha20-Poly1305"
    },
    layout: {
      title: "屏幕布局",
      description: (peerName) => `设置 ${peerName} 相对本机的方向`,
      canvasDirectionLabel: (direction) => `远端在${direction}`,
      localTag: "本机",
      peerTag: "远端",
      dragHint: "拖动远端屏幕到本机的任意一侧，松手后会自动吸附并保存方向",
      save: "保存布局",
      enabled: "已启用",
      disabled: "已停用",
      noPeerSelected: "未选择远端设备",
      noPeerName: "等待发现设备",
      summary: {
        direction: "方向",
        cornerGuard: "角落保护",
        cornerGuardValue: (px) => `${px} px`
      }
    },
    about: {
      tagline: "Cross-Device Mouse Companion",
      version: (version) => `版本 ${version}`,
      project: "项目",
      license: "许可证",
      licenseValue: "MIT"
    },
    statusStrip: {
      handoff: "鼠标切换",
      on: "开启",
      off: "关闭",
      rtt: "RTT",
      rttIdle: "—",
      discovered: (count) => `发现 ${count}`
    },
    languageToggleLabel: "切换到 English",
    states: {
      layoutDirections: {
        left: "左侧",
        right: "右侧",
        top: "上方",
        bottom: "下方"
      },
      permissions: {
        granted: "已授权",
        denied: "已拒绝",
        not_determined: "待设置",
        unsupported: "不适用",
        unknown: "未知"
      },
      sessions: {
        disconnected: "未连接",
        discovered: "已发现",
        pairing: "配对中",
        paired: "已配对",
        connecting: "连接中",
        connected_idle: "已连接",
        controlling_remote: "控制中",
        controlled_by_remote: "被控制",
        reconnecting: "重连中",
        error: "错误"
      },
      controlOwners: {
        local: "本机",
        remote: "远端",
        none: "无"
      },
      deviceStatuses: {
        available: "可连接",
        paired: "已配对",
        connected: "已连接",
        stale: "已过期"
      }
    }
  },
  en: {
    languageName: "English",
    loading: "Loading FlowLink",
    preferenceTitle: "Preferences",
    brand: "FlowLink",
    brandTagline: "LAN mouse handoff",
    tabs: {
      overview: "Overview",
      devices: "Devices",
      layout: "Layout",
      permissions: "Permissions",
      network: "Network",
      about: "About"
    },
    overview: {
      heroSubtitle: (sessionLabel) => `Session · ${sessionLabel}`,
      refresh: "Refresh",
      emergencyStop: "Emergency Stop",
      metricsTitle: "Runtime",
      metrics: {
        session: "Session",
        controlOwner: "Control",
        rtt: "Heartbeat",
        rttUnit: "ms",
        rttIdle: "Idle",
        discovered: "Discovered",
        trusted: "Trusted"
      },
      quickActionsTitle: "Quick Settings",
      handoffToggle: "Enable mouse handoff",
      handoffToggleHint: "Cross to remote when the cursor reaches the edge",
      discoveryToggle: "LAN auto-discovery",
      discoveryToggleHint: "Find peers via mDNS and UDP broadcast",
      autoReconnectToggle: "Auto reconnect",
      autoReconnectHint: "Reconnect after network glitches; remote control stays paused",
      liquidGlassToggle: "Liquid glass appearance",
      liquidGlassHint: "Use native macOS vibrancy / Windows Mica so the window blends with the desktop"
    },
    devices: {
      title: "Devices",
      discovered: (count) => `${count} discovered`,
      localBadge: "Local",
      empty: "No devices yet",
      emptyHint: "Make sure both devices share the same LAN with discovery enabled",
      pair: "Pair",
      connect: "Connect",
      forget: "Forget",
      addManual: "Add by IP"
    },
    permissions: {
      title: "Permissions",
      subtitle: "Without these, capture or injection will fail",
      accessibility: "Accessibility",
      accessibilityHint: "Required to inject mouse events into other apps",
      inputMonitoring: "Input Monitoring",
      inputMonitoringHint: "Required to observe local mouse movement",
      windowsInput: "Windows Input",
      windowsInputHint: "Normal user privileges are sufficient",
      open: "Open Settings"
    },
    network: {
      title: "Network",
      listenLabel: "Listen Port",
      discoveryProto: "Discovery",
      heartbeat: "Heartbeat",
      heartbeatUnit: "ms",
      rttLabel: "Current RTT",
      fallback: "Fallback",
      udpFallback: "UDP Broadcast",
      encryption: "Encryption",
      encryptionValue: "Noise XX · ChaCha20-Poly1305"
    },
    layout: {
      title: "Screen Layout",
      description: (peerName) => `Place ${peerName} relative to this device`,
      canvasDirectionLabel: (direction) => `Peer ${direction}`,
      localTag: "Local",
      peerTag: "Peer",
      dragHint: "Drag the peer screen to any side. It snaps and saves on release.",
      save: "Save Layout",
      enabled: "Active",
      disabled: "Disabled",
      noPeerSelected: "No peer selected",
      noPeerName: "Waiting for a device",
      summary: {
        direction: "Direction",
        cornerGuard: "Corner Guard",
        cornerGuardValue: (px) => `${px} px`
      }
    },
    about: {
      tagline: "Cross-Device Mouse Companion",
      version: (version) => `Version ${version}`,
      project: "Project",
      license: "License",
      licenseValue: "MIT"
    },
    statusStrip: {
      handoff: "Handoff",
      on: "On",
      off: "Off",
      rtt: "RTT",
      rttIdle: "—",
      discovered: (count) => `${count} peers`
    },
    languageToggleLabel: "Switch to 中文",
    states: {
      layoutDirections: {
        left: "Left",
        right: "Right",
        top: "Top",
        bottom: "Bottom"
      },
      permissions: {
        granted: "Granted",
        denied: "Denied",
        not_determined: "Not set",
        unsupported: "N/A",
        unknown: "Unknown"
      },
      sessions: {
        disconnected: "Disconnected",
        discovered: "Discovered",
        pairing: "Pairing",
        paired: "Paired",
        connecting: "Connecting",
        connected_idle: "Connected",
        controlling_remote: "Controlling",
        controlled_by_remote: "Controlled",
        reconnecting: "Reconnecting",
        error: "Error"
      },
      controlOwners: {
        local: "Local",
        remote: "Remote",
        none: "None"
      },
      deviceStatuses: {
        available: "Available",
        paired: "Paired",
        connected: "Connected",
        stale: "Stale"
      }
    }
  }
};

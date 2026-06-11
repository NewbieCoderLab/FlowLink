import type { LayoutDirection, PermissionState, SessionState, UiDevice } from "./types";

export type AppLanguage = "zh" | "en";

type DeviceStatus = UiDevice["status"];

export interface AppCopy {
  languageName: string;
  loading: string;
  heroEyebrow: string;
  heroDescription: string;
  languageToggleLabel: string;
  status: {
    session: string;
    controlOwner: string;
    refresh: string;
    emergencyStop: string;
  };
  devices: {
    title: string;
    discovered: (count: number) => string;
    localBadge: string;
    empty: string;
  };
  permissions: {
    title: string;
    ready: string;
    needsSetup: string;
    accessibility: string;
    inputMonitoring: string;
    windowsInput: string;
  };
  layout: {
    title: string;
    enabled: string;
    disabled: string;
    noPeerSelected: string;
    description: (peerName: string) => string;
    save: string;
  };
  diagnostics: {
    title: string;
    subtitle: string;
    discoveredPeers: string;
    trustedPeers: string;
    layouts: string;
    configPath: string;
  };
  states: {
    layoutDirections: Record<LayoutDirection, string>;
    permissions: Record<PermissionState, string>;
    sessions: Record<SessionState, string>;
    controlOwners: Record<"local" | "remote" | "none", string>;
    deviceStatuses: Record<DeviceStatus, string>;
  };
}

export const copy: Record<AppLanguage, AppCopy> = {
  zh: {
    languageName: "中文",
    loading: "正在加载 FlowLink...",
    heroEyebrow: "跨设备控制 MVP",
    heroDescription:
      "面向局域网设备协同的控制面板，用于发现设备、检查权限，并准备 macOS 与 Windows 之间的鼠标切换。",
    languageToggleLabel: "切换到 English",
    status: {
      session: "会话",
      controlOwner: "控制权",
      refresh: "刷新",
      emergencyStop: "紧急停止"
    },
    devices: {
      title: "设备",
      discovered: (count) => `发现 ${count} 台设备`,
      localBadge: "本机",
      empty: "暂无发现设备"
    },
    permissions: {
      title: "权限",
      ready: "可注入输入",
      needsSetup: "需要设置",
      accessibility: "辅助功能",
      inputMonitoring: "输入监听",
      windowsInput: "Windows 输入"
    },
    layout: {
      title: "布局",
      enabled: "已启用",
      disabled: "已停用",
      noPeerSelected: "未选择设备",
      description: (peerName) => `设置 ${peerName} 相对本机的位置。`,
      save: "保存布局"
    },
    diagnostics: {
      title: "诊断",
      subtitle: "核心运行状态快照",
      discoveredPeers: "发现设备",
      trustedPeers: "可信设备",
      layouts: "布局配置",
      configPath: "配置路径"
    },
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
        not_determined: "未决定",
        unsupported: "不支持",
        unknown: "未知"
      },
      sessions: {
        disconnected: "未连接",
        discovered: "已发现",
        pairing: "配对中",
        paired: "已配对",
        connecting: "连接中",
        connected_idle: "已连接",
        controlling_remote: "控制远端",
        controlled_by_remote: "被远端控制",
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
    loading: "Loading FlowLink...",
    heroEyebrow: "Cross-device Control MVP",
    heroDescription:
      "A LAN-first control panel for discovering peers, checking permissions, and preparing mouse handoff between macOS and Windows.",
    languageToggleLabel: "Switch to 中文",
    status: {
      session: "Session",
      controlOwner: "Control owner",
      refresh: "Refresh",
      emergencyStop: "Emergency Stop"
    },
    devices: {
      title: "Devices",
      discovered: (count) => `${count} discovered`,
      localBadge: "Local",
      empty: "No devices discovered"
    },
    permissions: {
      title: "Permissions",
      ready: "Ready to inject",
      needsSetup: "Needs setup",
      accessibility: "Accessibility",
      inputMonitoring: "Input Monitoring",
      windowsInput: "Windows Input"
    },
    layout: {
      title: "Layout",
      enabled: "Enabled",
      disabled: "Disabled",
      noPeerSelected: "No peer selected",
      description: (peerName) => `Choose where ${peerName} sits relative to this device.`,
      save: "Save Layout"
    },
    diagnostics: {
      title: "Diagnostics",
      subtitle: "Core runtime snapshot",
      discoveredPeers: "Discovered Peers",
      trustedPeers: "Trusted Peers",
      layouts: "Layouts",
      configPath: "Config Path"
    },
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
        not_determined: "Not Determined",
        unsupported: "Unsupported",
        unknown: "Unknown"
      },
      sessions: {
        disconnected: "Disconnected",
        discovered: "Discovered",
        pairing: "Pairing",
        paired: "Paired",
        connecting: "Connecting",
        connected_idle: "Connected Idle",
        controlling_remote: "Controlling Remote",
        controlled_by_remote: "Controlled By Remote",
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


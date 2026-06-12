import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { UiAppStatus, UiLayoutConfig, UiScreenTopology } from "./types";

const isTauriRuntime = "__TAURI_INTERNALS__" in window;

const mockStatus: UiAppStatus = {
  localDevice: {
    deviceId: "local-demo-device",
    name: "FlowLink Demo Host",
    os: "unknown",
    arch: "unknown",
    appVersion: "0.1.0",
    protocolVersion: 1,
    addressLabel: "127.0.0.1:42424",
    status: "connected",
    lastSeenLabel: "just now"
  },
  permission: {
    accessibility: "unknown",
    inputMonitoring: "unknown",
    screenRecording: "unsupported",
    windowsInput: "unknown",
    windowsIntegrityLevel: null,
    canCaptureMouse: false,
    canInjectMouse: false,
    updatedAtMs: Date.now()
  },
  session: {
    state: "disconnected",
    controlOwner: "local",
    peerName: null,
    lastHeartbeatRttMs: null,
    connectedSinceMs: null,
    updatedAtMs: Date.now()
  },
  discoveredDevices: [
    {
      deviceId: "peer-demo-device",
      name: "Office Windows PC",
      os: "windows",
      arch: "x86_64",
      appVersion: "0.1.0",
      protocolVersion: 1,
      addressLabel: "192.168.1.42:42424",
      status: "available",
      lastSeenLabel: "3s ago"
    }
  ],
  savedLayouts: [
    {
      peerId: "peer-demo-device",
      direction: "right",
      enabled: true
    }
  ],
  diagnostics: {
    discoveredPeerCount: 1,
    trustedPeerCount: 0,
    layoutCount: 1,
    configPath: "demo-mode"
  }
};

const mockScreenTopology: UiScreenTopology = {
  displays: [],
  virtualBounds: {
    x: 0,
    y: 0,
    width: 0,
    height: 0
  }
};

export async function getAppStatus(): Promise<UiAppStatus> {
  try {
    return await invoke<UiAppStatus>("get_app_status");
  } catch {
    return mockStatus;
  }
}

export async function saveLayout(layout: UiLayoutConfig): Promise<void> {
  try {
    await invoke("save_layout", { layout });
  } catch {
    mockStatus.savedLayouts = [layout];
  }
}

export async function getScreenTopology(): Promise<UiScreenTopology> {
  try {
    return await invoke<UiScreenTopology>("get_screen_topology");
  } catch {
    return mockScreenTopology;
  }
}

export async function disconnectPeer(): Promise<void> {
  try {
    await invoke("disconnect");
  } catch {
    mockStatus.session.state = "disconnected";
    mockStatus.session.peerName = null;
  }
}

export async function openPermissionSettings(permission: string): Promise<void> {
  try {
    await invoke("open_permission_settings", { permission });
  } catch {
    // Browser preview has no system settings integration.
  }
}

export async function probePeerIp(ip: string): Promise<void> {
  try {
    await invoke("probe_peer_ip", { ip });
  } catch {
    if (isTauriRuntime) {
      throw new Error("Failed to send manual discovery probe");
    }

    const peerId = `manual-${ip}`;
    if (!mockStatus.discoveredDevices.some((device) => device.deviceId === peerId)) {
      mockStatus.discoveredDevices = [
        {
          deviceId: peerId,
          name: `Manual ${ip}`,
          os: "unknown",
          arch: "unknown",
          appVersion: "0.1.0",
          protocolVersion: 1,
          addressLabel: `${ip}:42424`,
          status: "available",
          lastSeenLabel: "just now"
        },
        ...mockStatus.discoveredDevices
      ];
    }
  }
}

export async function listenPermissionUpdates(
  handler: () => void
): Promise<UnlistenFn> {
  try {
    return await listen("permission:updated", handler);
  } catch {
    return () => {};
  }
}

export async function listenDeviceDiscoveryUpdates(
  handler: () => void
): Promise<UnlistenFn> {
  try {
    const unlistenDiscovered = await listen("device:discovered", handler);
    const unlistenStale = await listen("device:stale", handler);
    return () => {
      unlistenDiscovered();
      unlistenStale();
    };
  } catch {
    return () => {};
  }
}

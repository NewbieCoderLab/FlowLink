import { useEffect, useMemo, useState } from "react";
import { Effect, EffectState, getCurrentWindow } from "@tauri-apps/api/window";
import { useAppStatus } from "../hooks/useAppStatus";
import { LayoutCanvas } from "../components/LayoutCanvas";
import { disconnectPeer, openPermissionSettings, saveLayout } from "./tauri";
import appIcon from "../assets/app-icon.png";
import { type AppLanguage, copy } from "./i18n";
import type { LayoutDirection, OsType, UiDevice } from "./types";

const preferenceTabs = [
  { key: "overview", icon: "overview" },
  { key: "devices", icon: "devices" },
  { key: "layout", icon: "layout" },
  { key: "permissions", icon: "permissions" },
  { key: "network", icon: "network" },
  { key: "about", icon: "about" }
] as const;

type PreferenceTabKey = (typeof preferenceTabs)[number]["key"];

const osLabel: Record<OsType, string> = {
  macos: "macOS",
  windows: "Windows",
  unknown: "—"
};

function deviceGlyphLetters(name: string): string {
  const trimmed = name.trim();
  if (!trimmed) return "··";
  const parts = trimmed.split(/[\s_-]+/).filter(Boolean);
  if (parts.length === 1) {
    return parts[0]!.slice(0, 2).toUpperCase();
  }
  return (parts[0]![0]! + parts[1]![0]!).toUpperCase();
}

function describePeer(device: UiDevice): string {
  const os = osLabel[device.os] ?? "";
  return [os, device.addressLabel].filter(Boolean).join(" · ");
}

const isTauri = "__TAURI_INTERNALS__" in (globalThis as Record<string, unknown>);

function pickEffectsForOs(): Effect[] {
  if (typeof navigator === "undefined") return [Effect.Tooltip];
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) {
    // Mos uses NSVisualEffectMaterial.toolTip for the same translucent feel.
    return [Effect.Tooltip];
  }
  if (ua.includes("windows")) {
    // Mica is Windows 11 only; Tauri falls back to Acrylic on older systems.
    return [Effect.Mica, Effect.Acrylic];
  }
  return [];
}

  async function applyLiquidGlass(enabled: boolean) {
  if (!isTauri) return;
  try {
    const win = getCurrentWindow();
    if (enabled) {
      const effects = pickEffectsForOs();
      if (effects.length === 0) {
        await win.clearEffects();
        return;
      }
      await win.setEffects({ effects, state: EffectState.FollowsWindowActiveState });
    } else {
      await win.clearEffects();
    }
  } catch {
    // Ignore on platforms where the API is unsupported (Linux). The CSS
    // fallback still produces a readable window.
  }
}

export function App() {
  const { status, loading, refresh } = useAppStatus();
  const [language, setLanguage] = useState<AppLanguage>("zh");
  const [activeTab, setActiveTab] = useState<PreferenceTabKey>("overview");
  const [layoutDirection, setLayoutDirection] = useState<LayoutDirection>("right");
  const [handoffEnabled, setHandoffEnabled] = useState(true);
  const [discoveryEnabled, setDiscoveryEnabled] = useState(true);
  const [autoReconnect, setAutoReconnect] = useState(true);
  const [liquidGlass, setLiquidGlass] = useState(true);
  const t = copy[language];

  useEffect(() => {
    if (!status) return;
    const saved = status.savedLayouts[0];
    if (saved && saved.direction !== layoutDirection) {
      setLayoutDirection(saved.direction);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status?.savedLayouts]);

  useEffect(() => {
    const root = document.getElementById("root");
    if (root) {
      root.classList.toggle("liquid-glass", liquidGlass);
      root.classList.toggle("solid-bg", !liquidGlass);
    }
    void applyLiquidGlass(liquidGlass);
  }, [liquidGlass]);

  const peerDevice = useMemo<UiDevice | null>(() => {
    if (!status) return null;
    const savedPeerId = status.savedLayouts[0]?.peerId;
    if (savedPeerId) {
      const match = status.discoveredDevices.find((device) => device.deviceId === savedPeerId);
      if (match) return match;
    }
    return status.discoveredDevices[0] ?? null;
  }, [status]);

  async function handleOpenPermission(permission: "accessibility" | "input_monitoring") {
    await openPermissionSettings(permission);
    await refresh();
  }

  if (loading || !status) {
    return <div className="loading-state">{t.loading}</div>;
  }

  const nextLanguage: AppLanguage = language === "zh" ? "en" : "zh";
  const sessionLabel = t.states.sessions[status.session.state];
  const controlOwnerLabel = t.states.controlOwners[status.session.controlOwner];
  const windowsIntegrityHint =
    t.permissions.windowsIntegrityHint?.(status.permission.windowsIntegrityLevel) ??
    `Integrity level: ${status.permission.windowsIntegrityLevel ?? "unknown"}; elevated windows may be blocked by Windows UIPI`;
  const rttValue =
    status.session.lastHeartbeatRttMs != null
      ? `${status.session.lastHeartbeatRttMs} ${t.overview.metrics.rttUnit}`
      : t.overview.metrics.rttIdle;

  const handleDirectionChange = (direction: LayoutDirection) => {
    setLayoutDirection(direction);
    if (peerDevice) {
      void saveLayout({
        peerId: peerDevice.deviceId,
        direction,
        enabled: true
      });
    }
  };

  const handleEmergencyStop = () => {
    void disconnectPeer().then(() => {
      void refresh();
    });
  };

  return (
    <main className="app-shell" data-tab={activeTab} lang={language === "zh" ? "zh-CN" : "en"}>
      <div className="title-drag" data-tauri-drag-region aria-hidden="true" />
      <nav
        className="preference-tabs"
        aria-label={t.preferenceTitle}
        data-tauri-drag-region
      >
        {preferenceTabs.map((tab) => (
          <button
            key={tab.key}
            type="button"
            className={activeTab === tab.key ? "preference-tab active" : "preference-tab"}
            aria-pressed={activeTab === tab.key}
            onClick={() => setActiveTab(tab.key)}
          >
            <span className={`tab-icon ${tab.icon}`} aria-hidden="true">
              {tab.key === "overview" ? <img src={appIcon} alt="" /> : null}
            </span>
            {t.tabs[tab.key]}
          </button>
        ))}
      </nav>

      <div className="tab-panel">
        {activeTab === "overview" ? (
          <>
            <div className="hero">
              <span className="hero-mark" aria-hidden="true" />
              <div className="hero-info">
                <h2>{status.localDevice.name}</h2>
                <p>{t.overview.heroSubtitle(sessionLabel)}</p>
              </div>
              <div className="hero-actions">
                <button type="button" className="btn" onClick={() => void refresh()}>
                  {t.overview.refresh}
                </button>
                <button type="button" className="btn btn-danger" onClick={handleEmergencyStop}>
                  {t.overview.emergencyStop}
                </button>
              </div>
            </div>
            <div className="row">
              <span className="row-label">{t.overview.metrics.session}</span>
              <span className="row-value">
                <small>{controlOwnerLabel} · {rttValue}</small>
              </span>
              <span className={`pill ${status.session.state}`}>
                <span className="pill-dot" />
                {sessionLabel}
              </span>
            </div>
            <div className="row">
              <span className="row-label">{t.overview.handoffToggle}</span>
              <span className="row-value">
                <small>{t.overview.handoffToggleHint}</small>
              </span>
              <label className="toggle">
                <input
                  type="checkbox"
                  checked={handoffEnabled}
                  onChange={(e) => setHandoffEnabled(e.target.checked)}
                />
                <span className="toggle-track" />
              </label>
            </div>
            <div className="row">
              <span className="row-label">{t.overview.discoveryToggle}</span>
              <span className="row-value">
                <small>{t.overview.discoveryToggleHint}</small>
              </span>
              <label className="toggle">
                <input
                  type="checkbox"
                  checked={discoveryEnabled}
                  onChange={(e) => setDiscoveryEnabled(e.target.checked)}
                />
                <span className="toggle-track" />
              </label>
            </div>
            <div className="row">
              <span className="row-label">{t.overview.autoReconnectToggle}</span>
              <span className="row-value">
                <small>{t.overview.autoReconnectHint}</small>
              </span>
              <label className="toggle">
                <input
                  type="checkbox"
                  checked={autoReconnect}
                  onChange={(e) => setAutoReconnect(e.target.checked)}
                />
                <span className="toggle-track" />
              </label>
            </div>
            <div className="row">
              <span className="row-label">{t.overview.liquidGlassToggle}</span>
              <span className="row-value">
                <small>{t.overview.liquidGlassHint}</small>
              </span>
              <label className="toggle">
                <input
                  type="checkbox"
                  checked={liquidGlass}
                  onChange={(e) => setLiquidGlass(e.target.checked)}
                />
                <span className="toggle-track" />
              </label>
            </div>
          </>
        ) : null}

        {activeTab === "devices" ? (
          <>
            <div className="section-title">
              <span>{t.devices.localBadge}</span>
            </div>
            <div className="device-row" style={{ cursor: "default" }}>
              <span className={`device-glyph ${status.localDevice.os}`}>
                {deviceGlyphLetters(status.localDevice.name)}
              </span>
              <span className="device-meta">
                <strong>{status.localDevice.name}</strong>
                <small>{describePeer(status.localDevice)}</small>
              </span>
              <span className={`pill ${status.localDevice.status}`}>
                <span className="pill-dot" />
                {t.states.deviceStatuses[status.localDevice.status]}
              </span>
            </div>
            <div className="section-title">
              <span>{t.devices.title}</span>
              <small>{t.devices.discovered(status.discoveredDevices.length)}</small>
            </div>
            {status.discoveredDevices.length === 0 ? (
              <div className="device-empty">
                <span>{t.devices.empty}</span>
                <small>{t.devices.emptyHint}</small>
              </div>
            ) : (
              <div className="device-list">
                {status.discoveredDevices.map((device) => (
                  <button key={device.deviceId} type="button" className="device-row">
                    <span className={`device-glyph ${device.os}`}>{deviceGlyphLetters(device.name)}</span>
                    <span className="device-meta">
                      <strong>{device.name}</strong>
                      <small>{describePeer(device)}</small>
                    </span>
                    <span className="row-action">
                      <span className={`pill ${device.status}`}>
                        <span className="pill-dot" />
                        {t.states.deviceStatuses[device.status]}
                      </span>
                      <button type="button" className="btn btn-primary">
                        {device.status === "paired" || device.status === "connected"
                          ? t.devices.connect
                          : t.devices.pair}
                      </button>
                    </span>
                  </button>
                ))}
              </div>
            )}
          </>
        ) : null}

        {activeTab === "layout" ? (
          <LayoutCanvas
            copy={t}
            localName={status.localDevice.name}
            localOsLabel={osLabel[status.localDevice.os]}
            peerName={peerDevice?.name ?? t.layout.noPeerName}
            peerOsLabel={peerDevice ? osLabel[peerDevice.os] : "—"}
            direction={layoutDirection}
            onDirectionChange={handleDirectionChange}
          />
        ) : null}

        {activeTab === "permissions" ? (
          <>
            <div className="row">
              <span className="row-label">{t.permissions.accessibility}</span>
              <span className="row-value">
                <small>{t.permissions.accessibilityHint}</small>
              </span>
              <span className="row-action">
                <span className={`pill ${status.permission.accessibility}`}>
                  <span className="pill-dot" />
                  {t.states.permissions[status.permission.accessibility]}
                </span>
                <button
                  type="button"
                  className="btn"
                  onClick={() => void handleOpenPermission("accessibility")}
                >
                  {t.permissions.open}
                </button>
              </span>
            </div>
            <div className="row">
              <span className="row-label">{t.permissions.inputMonitoring}</span>
              <span className="row-value">
                <small>{t.permissions.inputMonitoringHint}</small>
              </span>
              <span className="row-action">
                <span className={`pill ${status.permission.inputMonitoring}`}>
                  <span className="pill-dot" />
                  {t.states.permissions[status.permission.inputMonitoring]}
                </span>
                <button
                  type="button"
                  className="btn"
                  onClick={() => void handleOpenPermission("input_monitoring")}
                >
                  {t.permissions.open}
                </button>
              </span>
            </div>
            <div className="row">
              <span className="row-label">{t.permissions.windowsInput}</span>
              <span className="row-value">
                <small>{t.permissions.windowsInputHint}</small>
                <small>{windowsIntegrityHint}</small>
              </span>
              <span className="row-action">
                <span className={`pill ${status.permission.windowsInput}`}>
                  <span className="pill-dot" />
                  {t.states.permissions[status.permission.windowsInput]}
                </span>
                <button type="button" className="btn" disabled>
                  {t.permissions.open}
                </button>
              </span>
            </div>
          </>
        ) : null}

        {activeTab === "network" ? (
          <>
            <div className="row">
              <span className="row-label">{t.network.discoveryProto}</span>
              <span className="row-value">
                <strong>mDNS</strong>
                <small>_mac22win._tcp.local</small>
              </span>
              <span className="pill discovered">
                <span className="pill-dot" />
                {t.statusStrip.discovered(status.discoveredDevices.length)}
              </span>
            </div>
            <div className="row">
              <span className="row-label">{t.network.fallback}</span>
              <span className="row-value">
                <strong>{t.network.udpFallback}</strong>
                <small>port 42425</small>
              </span>
              <span className={`pill ${discoveryEnabled ? "available" : "disconnected"}`}>
                <span className="pill-dot" />
                {discoveryEnabled ? t.statusStrip.on : t.statusStrip.off}
              </span>
            </div>
            <div className="row">
              <span className="row-label">{t.network.listenLabel}</span>
              <span className="row-value">
                <strong>TCP 42424</strong>
                <small>TCP_NODELAY · 64 KiB frames</small>
              </span>
              <span className="pill connected_idle">
                <span className="pill-dot" />
                Active
              </span>
            </div>
            <div className="row">
              <span className="row-label">{t.network.heartbeat}</span>
              <span className="row-value">
                <strong>1000 {t.network.heartbeatUnit}</strong>
                <small>timeout 3000 ms</small>
              </span>
              <span className="pill connected_idle">
                <span className="pill-dot" />
                {rttValue}
              </span>
            </div>
            <div className="row">
              <span className="row-label">{t.network.encryption}</span>
              <span className="row-value">
                <strong>{t.network.encryptionValue}</strong>
                <small>Ed25519 pinned · forward secrecy</small>
              </span>
              <span className="pill granted">
                <span className="pill-dot" />
                {t.statusStrip.on}
              </span>
            </div>
          </>
        ) : null}

        {activeTab === "about" ? (
          <div className="about-panel">
            <img src={appIcon} alt="" />
            <h2>{t.brand}</h2>
            <span className="about-tagline">{t.about.tagline}</span>
            <p>{t.about.version(status.localDevice.appVersion)}</p>
            <p style={{ color: "var(--fl-text-tertiary)", fontSize: 10 }}>
              protocol v{status.localDevice.protocolVersion} · {t.about.licenseValue}
            </p>
            <button
              type="button"
              className="btn"
              style={{ marginTop: 6 }}
              onClick={() => setLanguage(nextLanguage)}
              aria-label={t.languageToggleLabel}
            >
              {copy[nextLanguage].languageName}
            </button>
          </div>
        ) : null}
      </div>
    </main>
  );
}

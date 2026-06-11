import { useState } from "react";
import { DeviceList } from "../components/DeviceList";
import { DiagnosticsPanel } from "../components/DiagnosticsPanel";
import { LayoutEditor } from "../components/LayoutEditor";
import { PermissionPanel } from "../components/PermissionPanel";
import { StatusBar } from "../components/StatusBar";
import { useAppStatus } from "../hooks/useAppStatus";
import { type AppLanguage, copy } from "./i18n";
import { disconnectPeer, saveLayout } from "./tauri";
import type { LayoutDirection } from "./types";

export function App() {
  const { status, loading, refresh } = useAppStatus();
  const [language, setLanguage] = useState<AppLanguage>("zh");
  const [layoutDirection, setLayoutDirection] = useState<LayoutDirection>("right");
  const t = copy[language];

  if (loading || !status) {
    return <div className="app-shell loading-state">{t.loading}</div>;
  }

  const primaryPeer = status.discoveredDevices[0];
  const nextLanguage: AppLanguage = language === "zh" ? "en" : "zh";

  return (
    <main className="app-shell" lang={language === "zh" ? "zh-CN" : "en"}>
      <section className="hero-panel">
        <div>
          <p className="eyebrow">{t.heroEyebrow}</p>
          <h1>FlowLink</h1>
          <p className="hero-copy">{t.heroDescription}</p>
        </div>
        <button
          type="button"
          className="language-toggle"
          aria-label={t.languageToggleLabel}
          onClick={() => setLanguage(nextLanguage)}
        >
          {copy[nextLanguage].languageName}
        </button>
      </section>

      <StatusBar
        copy={t}
        deviceName={status.localDevice.name}
        session={status.session}
        onRefresh={refresh}
        onDisconnect={disconnectPeer}
      />

      <div className="content-grid">
        <PermissionPanel copy={t} permission={status.permission} />
        <DeviceList copy={t} localDevice={status.localDevice} devices={status.discoveredDevices} />
        <LayoutEditor
          copy={t}
          peerName={primaryPeer?.name ?? t.layout.noPeerSelected}
          direction={layoutDirection}
          enabled={status.savedLayouts[0]?.enabled ?? true}
          onDirectionChange={setLayoutDirection}
          onSave={async () => {
            if (!primaryPeer) {
              return;
            }

            await saveLayout({
              peerId: primaryPeer.deviceId,
              direction: layoutDirection,
              enabled: true
            });
            await refresh();
          }}
        />
        <DiagnosticsPanel copy={t} diagnostics={status.diagnostics} />
      </div>
    </main>
  );
}

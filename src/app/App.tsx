import { useState } from "react";
import { DeviceList } from "../components/DeviceList";
import { DiagnosticsPanel } from "../components/DiagnosticsPanel";
import { LayoutEditor } from "../components/LayoutEditor";
import { PermissionPanel } from "../components/PermissionPanel";
import { StatusBar } from "../components/StatusBar";
import { useAppStatus } from "../hooks/useAppStatus";
import { disconnectPeer, saveLayout } from "./tauri";
import type { LayoutDirection } from "./types";

export function App() {
  const { status, loading, refresh } = useAppStatus();
  const [layoutDirection, setLayoutDirection] = useState<LayoutDirection>("right");

  if (loading || !status) {
    return <div className="app-shell loading-state">Loading FlowLink...</div>;
  }

  const primaryPeer = status.discoveredDevices[0];

  return (
    <main className="app-shell">
      <section className="hero-panel">
        <p className="eyebrow">Cross-device Control MVP</p>
        <h1>FlowLink</h1>
        <p className="hero-copy">
          A LAN-first control surface for discovering peers, validating permissions, and preparing
          reliable mouse handoff between macOS and Windows.
        </p>
      </section>

      <StatusBar
        deviceName={status.localDevice.name}
        session={status.session}
        onRefresh={refresh}
        onDisconnect={disconnectPeer}
      />

      <div className="content-grid">
        <PermissionPanel permission={status.permission} />
        <DeviceList localDevice={status.localDevice} devices={status.discoveredDevices} />
        <LayoutEditor
          peerName={primaryPeer?.name ?? "No peer selected"}
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
        <DiagnosticsPanel diagnostics={status.diagnostics} />
      </div>
    </main>
  );
}


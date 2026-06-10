import type { UiSessionStatus } from "../app/types";

interface StatusBarProps {
  deviceName: string;
  session: UiSessionStatus;
  onRefresh: () => Promise<void>;
  onDisconnect: () => Promise<void>;
}

export function StatusBar({ deviceName, session, onRefresh, onDisconnect }: StatusBarProps) {
  return (
    <section className="status-bar">
      <div>
        <strong>{deviceName}</strong>
        <span>
          Session: {session.state} / Control owner: {session.controlOwner}
        </span>
      </div>
      <div className="status-actions">
        <button type="button" className="ghost-button" onClick={() => void onRefresh()}>
          Refresh
        </button>
        <button type="button" className="ghost-button" onClick={() => void onDisconnect()}>
          Emergency Stop
        </button>
      </div>
    </section>
  );
}


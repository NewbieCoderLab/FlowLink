import type { AppCopy } from "../app/i18n";
import type { UiSessionStatus } from "../app/types";

interface StatusBarProps {
  copy: AppCopy;
  deviceName: string;
  session: UiSessionStatus;
  onRefresh: () => Promise<void>;
  onDisconnect: () => Promise<void>;
}

export function StatusBar({ copy, deviceName, session, onRefresh, onDisconnect }: StatusBarProps) {
  return (
    <section className="status-bar">
      <div>
        <strong>{deviceName}</strong>
        <span>
          {copy.status.session}: {copy.states.sessions[session.state]} / {copy.status.controlOwner}:{" "}
          {copy.states.controlOwners[session.controlOwner]}
        </span>
      </div>
      <div className="status-actions">
        <button type="button" className="ghost-button" onClick={() => void onRefresh()}>
          {copy.status.refresh}
        </button>
        <button type="button" className="ghost-button" onClick={() => void onDisconnect()}>
          {copy.status.emergencyStop}
        </button>
      </div>
    </section>
  );
}

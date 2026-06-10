import type { UiPermissionStatus } from "../app/types";

interface PermissionPanelProps {
  permission: UiPermissionStatus;
}

export function PermissionPanel({ permission }: PermissionPanelProps) {
  const rows = [
    ["Accessibility", permission.accessibility],
    ["Input Monitoring", permission.inputMonitoring],
    ["Windows Input", permission.windowsInput]
  ] as const;

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Permissions</h2>
        <span>{permission.canInjectMouse ? "Ready to inject" : "Needs setup"}</span>
      </div>
      <div className="permission-list">
        {rows.map(([label, value]) => (
          <div key={label} className="permission-row">
            <span>{label}</span>
            <strong className={`permission-state ${value}`}>{value}</strong>
          </div>
        ))}
      </div>
    </section>
  );
}


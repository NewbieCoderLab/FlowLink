import type { AppCopy } from "../app/i18n";
import type { UiPermissionStatus } from "../app/types";

interface PermissionPanelProps {
  copy: AppCopy;
  permission: UiPermissionStatus;
}

export function PermissionPanel({ copy, permission }: PermissionPanelProps) {
  const rows = [
    [copy.permissions.accessibility, permission.accessibility],
    [copy.permissions.inputMonitoring, permission.inputMonitoring],
    [copy.permissions.windowsInput, permission.windowsInput]
  ] as const;

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>{copy.permissions.title}</h2>
        <span>{permission.canInjectMouse ? copy.permissions.ready : copy.permissions.needsSetup}</span>
      </div>
      <div className="permission-list">
        {rows.map(([label, value]) => (
          <div key={label} className="permission-row">
            <span>{label}</span>
            <strong className={`permission-state ${value}`}>{copy.states.permissions[value]}</strong>
          </div>
        ))}
      </div>
    </section>
  );
}

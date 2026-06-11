import type { AppCopy } from "../app/i18n";
import type { UiDevice } from "../app/types";

interface DeviceListProps {
  copy: AppCopy;
  localDevice: UiDevice;
  devices: UiDevice[];
}

export function DeviceList({ copy, localDevice, devices }: DeviceListProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>{copy.devices.title}</h2>
        <span>{copy.devices.discovered(devices.length)}</span>
      </div>
      <div className="device-card local">
        <strong>{localDevice.name}</strong>
        <span>{copy.devices.localBadge}</span>
        <span>{localDevice.os}</span>
        <span>{localDevice.addressLabel}</span>
      </div>
      <div className="device-list">
        {devices.length === 0 ? (
          <div className="empty-state">{copy.devices.empty}</div>
        ) : (
          devices.map((device) => (
            <div key={device.deviceId} className="device-card">
              <strong>{device.name}</strong>
              <span>
                {device.os} / {device.arch}
              </span>
              <span>{device.addressLabel}</span>
              <span className={`status-pill ${device.status}`}>
                {copy.states.deviceStatuses[device.status]}
              </span>
              <small>{device.lastSeenLabel}</small>
            </div>
          ))
        )}
      </div>
    </section>
  );
}

import type { UiDevice } from "../app/types";

interface DeviceListProps {
  localDevice: UiDevice;
  devices: UiDevice[];
}

export function DeviceList({ localDevice, devices }: DeviceListProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Devices</h2>
        <span>{devices.length} discovered</span>
      </div>
      <div className="device-card local">
        <strong>{localDevice.name}</strong>
        <span>{localDevice.os}</span>
        <span>{localDevice.addressLabel}</span>
      </div>
      <div className="device-list">
        {devices.map((device) => (
          <div key={device.deviceId} className="device-card">
            <strong>{device.name}</strong>
            <span>
              {device.os} / {device.arch}
            </span>
            <span>{device.addressLabel}</span>
            <span className={`status-pill ${device.status}`}>{device.status}</span>
            <small>{device.lastSeenLabel}</small>
          </div>
        ))}
      </div>
    </section>
  );
}


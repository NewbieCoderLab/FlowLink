import type { LayoutDirection } from "../app/types";

interface LayoutEditorProps {
  peerName: string;
  direction: LayoutDirection;
  enabled: boolean;
  onDirectionChange: (direction: LayoutDirection) => void;
  onSave: () => Promise<void>;
}

const directions: LayoutDirection[] = ["left", "right", "top", "bottom"];

export function LayoutEditor({
  peerName,
  direction,
  enabled,
  onDirectionChange,
  onSave
}: LayoutEditorProps) {
  return (
    <section className="panel layout-panel">
      <div className="panel-header">
        <h2>Layout</h2>
        <span>{enabled ? "Enabled" : "Disabled"}</span>
      </div>
      <p className="layout-copy">Choose where {peerName} sits relative to this device.</p>
      <div className="layout-selector">
        {directions.map((item) => (
          <button
            key={item}
            type="button"
            className={item === direction ? "layout-button active" : "layout-button"}
            onClick={() => onDirectionChange(item)}
          >
            {item}
          </button>
        ))}
      </div>
      <button type="button" className="primary-button" onClick={() => void onSave()}>
        Save Layout
      </button>
    </section>
  );
}


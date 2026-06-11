import type { AppCopy } from "../app/i18n";
import type { LayoutDirection } from "../app/types";

interface LayoutEditorProps {
  copy: AppCopy;
  peerName: string;
  direction: LayoutDirection;
  enabled: boolean;
  onDirectionChange: (direction: LayoutDirection) => void;
  onSave: () => Promise<void>;
}

const directions: LayoutDirection[] = ["left", "right", "top", "bottom"];

export function LayoutEditor({
  copy,
  peerName,
  direction,
  enabled,
  onDirectionChange,
  onSave
}: LayoutEditorProps) {
  return (
    <section className="panel layout-panel">
      <div className="panel-header">
        <h2>{copy.layout.title}</h2>
        <span>{enabled ? copy.layout.enabled : copy.layout.disabled}</span>
      </div>
      <p className="layout-copy">{copy.layout.description(peerName)}</p>
      <div className="layout-selector">
        {directions.map((item) => (
          <button
            key={item}
            type="button"
            className={item === direction ? "layout-button active" : "layout-button"}
            onClick={() => onDirectionChange(item)}
          >
            {copy.states.layoutDirections[item]}
          </button>
        ))}
      </div>
      <button type="button" className="primary-button" onClick={() => void onSave()}>
        {copy.layout.save}
      </button>
    </section>
  );
}

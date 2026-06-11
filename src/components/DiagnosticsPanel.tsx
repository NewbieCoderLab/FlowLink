import type { AppCopy } from "../app/i18n";
import type { UiDiagnostics } from "../app/types";

interface DiagnosticsPanelProps {
  copy: AppCopy;
  diagnostics: UiDiagnostics;
}

export function DiagnosticsPanel({ copy, diagnostics }: DiagnosticsPanelProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>{copy.diagnostics.title}</h2>
        <span>{copy.diagnostics.subtitle}</span>
      </div>
      <dl className="diagnostics-grid">
        <div>
          <dt>{copy.diagnostics.discoveredPeers}</dt>
          <dd>{diagnostics.discoveredPeerCount}</dd>
        </div>
        <div>
          <dt>{copy.diagnostics.trustedPeers}</dt>
          <dd>{diagnostics.trustedPeerCount}</dd>
        </div>
        <div>
          <dt>{copy.diagnostics.layouts}</dt>
          <dd>{diagnostics.layoutCount}</dd>
        </div>
        <div>
          <dt>{copy.diagnostics.configPath}</dt>
          <dd>{diagnostics.configPath}</dd>
        </div>
      </dl>
    </section>
  );
}

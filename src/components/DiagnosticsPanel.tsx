import type { UiDiagnostics } from "../app/types";

interface DiagnosticsPanelProps {
  diagnostics: UiDiagnostics;
}

export function DiagnosticsPanel({ diagnostics }: DiagnosticsPanelProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Diagnostics</h2>
        <span>Core runtime snapshot</span>
      </div>
      <dl className="diagnostics-grid">
        <div>
          <dt>Discovered Peers</dt>
          <dd>{diagnostics.discoveredPeerCount}</dd>
        </div>
        <div>
          <dt>Trusted Peers</dt>
          <dd>{diagnostics.trustedPeerCount}</dd>
        </div>
        <div>
          <dt>Layouts</dt>
          <dd>{diagnostics.layoutCount}</dd>
        </div>
        <div>
          <dt>Config Path</dt>
          <dd>{diagnostics.configPath}</dd>
        </div>
      </dl>
    </section>
  );
}


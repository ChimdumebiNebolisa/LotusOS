import { useEffect, useState } from "react";
import { api } from "../api";
import type { Checkpoint, Drift, Finding, LedgerEvent, StatusReport, WorkspaceListEntry } from "../types";
import { STATE_LABELS } from "../types";

const stateClass = (state: string) => `state-chip state-${state}`;

const StateChip = ({ state }: { state: string }) => (
  <span className={stateClass(state)}>{STATE_LABELS[state] ?? state.toUpperCase()}</span>
);

type Props = {
  workspace: WorkspaceListEntry;
  status: StatusReport | null;
  onAction: (action: "start" | "stop" | "restart" | "doctor" | "checkpoint") => void;
  busy: string | null;
};

export function WorkspaceDetail({ workspace, status, onAction, busy }: Props) {
  const [doctor, setDoctor] = useState<Finding[] | null>(null);
  const [events, setEvents] = useState<LedgerEvent[]>([]);
  const [logs, setLogs] = useState<string[]>([]);
  const [checkpoints, setCheckpoints] = useState<Checkpoint[]>([]);
  const [restorePick, setRestorePick] = useState<string>("");
  const [drift, setDrift] = useState<Drift[] | null>(null);

  const selector = workspace.name;

  useEffect(() => {
    let alive = true;
    api.events(selector, 25).then((e) => alive && setEvents(e)).catch(() => alive && setEvents([]));
    return () => { alive = false; };
  }, [selector, status?.state, status?.updated_at_ms]);

  useEffect(() => {
    let alive = true;
    api.logs(selector, undefined, 40).then((l) => alive && setLogs(l)).catch(() => alive && setLogs([]));
    return () => { alive = false; };
  }, [selector, status?.updated_at_ms]);

  useEffect(() => {
    let alive = true;
    api.checkpoints(selector).then((c) => alive && setCheckpoints(c)).catch(() => alive && setCheckpoints([]));
    return () => { alive = false; };
  }, [selector]);

  const runDoctor = async () => {
    try {
      setDoctor(await api.doctor(selector));
    } catch {
      setDoctor([]);
    }
  };

  const makeCheckpoint = async () => {
    const note = window.prompt("Checkpoint note (optional)") ?? undefined;
    await api.createCheckpoint(selector, note || undefined);
    setCheckpoints(await api.checkpoints(selector));
  };

  const previewRestore = async (id: string) => {
    setRestorePick(id);
    if (!id) { setDrift(null); return; }
    try {
      const preview = await api.restorePreview(selector, id);
      setDrift(preview.drift);
    } catch {
      setDrift([{ kind: "error", expected: "-", found: "checkpoint not readable" }]);
    }
  };

  const doRestore = async () => {
    if (!restorePick) return;
    if (!window.confirm("Stop the workspace and restart it from this checkpoint?")) return;
    await api.restore(selector, restorePick);
    setDrift(null);
    setRestorePick("");
  };

  return (
    <section className="detail">
      <header className="hero detail-hero">
        <div>
          <p className="eyebrow">Workspace</p>
          <h2>{workspace.name}</h2>
          <p className="description mono-small">{workspace.root}</p>
        </div>
        <div className="badge-row">
          <StateChip state={status?.state ?? "off"} />
          {workspace.trusted ? (
            workspace.manifest_drift ? <span className="trust-chip drift">MANIFEST CHANGED</span> : null
          ) : (
            <span className="trust-chip untrusted">UNTRUSTED</span>
          )}
        </div>
        <div className="action-row">
          <button type="button" disabled={busy !== null} onClick={() => onAction("start")}>Start</button>
          <button type="button" disabled={busy !== null} onClick={() => onAction("stop")}>Stop</button>
          <button type="button" disabled={busy !== null} onClick={() => onAction("restart")}>Restart</button>
          {!workspace.trusted || workspace.manifest_drift ? (
            <button
              type="button"
              className="accent"
              disabled={busy !== null}
              onClick={() => api.grantTrust(selector).then(() => window.location.reload())}
            >
              Review &amp; trust
            </button>
          ) : null}
        </div>
      </header>

      {status?.last_error ? (
        <article className="panel error-panel">
          <h3>Last error</h3>
          <p className="mono-small">{status.last_error}</p>
        </article>
      ) : null}

      <div className="grid">
        <article className="panel panel-wide">
          <div className="panel-heading">
            <h3>Processes</h3>
            <p className="panel-copy">{status?.processes.length ?? 0} declared</p>
          </div>
          {(status?.processes.length ?? 0) === 0 ? (
            <p>No process state recorded yet. Start the workspace to populate lifecycle data.</p>
          ) : (
            <table className="proc-table">
              <thead>
                <tr><th>Name</th><th>State</th><th>Health</th><th>PID</th><th>Restarts</th><th></th></tr>
              </thead>
              <tbody>
                {status!.processes.map((p) => (
                  <tr key={p.name}>
                    <td>{p.name}</td>
                    <td>{p.state}</td>
                    <td>{p.healthy === true ? "pass" : p.healthy === false ? "FAIL" : "-"}</td>
                    <td>{p.pid ?? "-"}</td>
                    <td>{p.restarts}</td>
                    <td className="mono-small">{p.detail ?? ""}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </article>

        <article className={`panel ${(status?.port_conflicts.length ?? 0) > 0 ? "warn-panel" : ""}`}>
          <h3>Ports</h3>
          {(status?.port_conflicts.length ?? 0) === 0 ? (
            <p>No conflicts reported.</p>
          ) : (
            <ul className="list">
              {status!.port_conflicts.map((c) => (
                <li key={c.port}>
                  <strong>{c.port}</strong> wanted by <code>{c.expected_owner}</code>, held by{" "}
                  {c.owner_name ?? "?"} (pid {c.owner_pid ?? "?"}). {c.remediation}
                </li>
              ))}
            </ul>
          )}
        </article>

        <article className="panel">
          <h3>Doctor</h3>
          <div className="action-row">
            <button type="button" onClick={runDoctor}>Run doctor</button>
          </div>
          {doctor ? (
            <table className="proc-table compact">
              <thead><tr><th>Check</th><th>Subject</th><th>Status</th><th>Detail</th></tr></thead>
              <tbody>
                {doctor.map((f, i) => (
                  <tr key={`${f.check}-${i}`}>
                    <td>{f.check}</td>
                    <td>{f.subject}</td>
                    <td><span className={`finding finding-${f.status}`}>{f.status}</span></td>
                    <td className="mono-small">{f.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p>Runs read-only environment checks. Env var values are never displayed.</p>
          )}
        </article>

        <article className="panel">
          <h3>Recent events</h3>
          <ul className="event-list">
            {events.map((e) => (
              <li key={e.seq}>
                <span className="mono-small">#{e.seq}</span>{" "}
                <span className="mono-small">{new Date(e.ts_ms).toLocaleTimeString()}</span>{" "}
                {e.kind}{e.process ? ` · ${e.process}` : ""}
              </li>
            ))}
            {events.length === 0 ? <li>No events yet.</li> : null}
          </ul>
        </article>

        <article className="panel panel-wide">
          <h3>Logs</h3>
          <pre className="log-view">{logs.length > 0 ? logs.join("\n") : "(no output captured)"}</pre>
        </article>

        <article className="panel panel-wide">
          <div className="panel-heading">
            <h3>Checkpoints</h3>
            <div className="action-row">
              <button type="button" disabled={busy !== null} onClick={makeCheckpoint}>Create checkpoint</button>
            </div>
          </div>
          {checkpoints.length === 0 ? (
            <p>None yet. A checkpoint records manifest hash, git position, and last health - not live memory.</p>
          ) : (
            <>
              <table className="proc-table">
                <thead><tr><th></th><th>ID</th><th>Taken</th><th>Branch</th><th>Commit</th><th>Note</th></tr></thead>
                <tbody>
                  {checkpoints.map((c) => (
                    <tr key={c.id}>
                      <td>
                        <input
                          type="radio"
                          name="checkpoint"
                          checked={restorePick === c.id}
                          onChange={() => previewRestore(c.id)}
                        />
                      </td>
                      <td className="mono-small">{c.id.slice(0, 18)}…</td>
                      <td>{new Date(c.created_at_ms).toLocaleString()}</td>
                      <td>{c.git_branch ?? "-"}</td>
                      <td className="mono-small">{c.git_commit?.slice(0, 8) ?? "-"}</td>
                      <td>{c.note ?? "-"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {drift ? (
                <div className="drift-box">
                  <p className="panel-copy">Restore drift report:</p>
                  {drift.length === 0 ? (
                    <p>No drift detected.</p>
                  ) : (
                    <ul className="list">
                      {drift.map((d, i) => (
                        <li key={i}><code>{d.kind}</code>: expected “{d.expected}”, found “{d.found}”</li>
                      ))}
                    </ul>
                  )}
                  <div className="action-row">
                    <button type="button" className="accent" disabled={busy !== null} onClick={doRestore}>
                      Restore selected checkpoint
                    </button>
                  </div>
                </div>
              ) : null}
            </>
          )}
        </article>
      </div>
    </section>
  );
}

import { useCallback, useEffect, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { api } from "./api";
import type { StatusReport, WorkspaceListEntry } from "./types";
import { STATE_LABELS } from "./types";
import { AddWorkspaceFlow } from "./components/AddWorkspaceFlow";
import { WorkspaceDetail } from "./components/WorkspaceDetail";

const stateClass = (state: string) => `state-chip state-${state}`;

export default function App() {
  const [workspaces, setWorkspaces] = useState<WorkspaceListEntry[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [status, setStatus] = useState<StatusReport | null>(null);
  const [view, setView] = useState<"list" | "add">("list");
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refreshList = useCallback(async () => {
    try {
      const list = await api.listWorkspaces();
      setWorkspaces(list);
      setSelected((current) => {
        if (current && list.some((w) => w.name === current)) return current;
        return list.length > 0 ? list[0].name : null;
      });
    } catch {
      setWorkspaces([]);
    }
  }, []);

  useEffect(() => {
    if (!isTauri()) return;
    refreshList();
  }, [refreshList]);

  // Poll the selected workspace status while it lives.
  useEffect(() => {
    if (!isTauri() || !selected) return;
    let alive = true;
    const tick = () => api.status(selected).then((s) => alive && setStatus(s)).catch(() => {});
    tick();
    const id = window.setInterval(tick, 1200);
    return () => { alive = false; window.clearInterval(id); };
  }, [selected]);

  const act = async (action: "start" | "stop" | "restart" | "doctor" | "checkpoint") => {
    if (!selected) return;
    setBusy(action);
    setNotice(null);
    try {
      if (action === "start") await api.startWorkspace(selected);
      if (action === "stop") await api.stopWorkspace(selected);
      if (action === "restart") await api.restartWorkspace(selected);
    } catch (e) {
      setNotice(String(e));
    } finally {
      setBusy(null);
    }
  };

  if (!isTauri()) {
    return (
      <div className="app-shell">
        <aside className="rail">
          <div className="brand-mark">LO</div>
          <div className="brand-copy">
            <p className="eyebrow">LotusOS</p>
            <h1>Shell</h1>
            <p className="caption">workspace runtime</p>
          </div>
        </aside>
        <main className="workspace">
          <header className="hero">
            <p className="eyebrow">Preview</p>
            <h2>Lotus Shell runs inside its desktop app.</h2>
            <p className="description">
              This browser preview has no access to local workspaces. Launch the packaged
              Tauri app to register and operate workspaces.
            </p>
          </header>
        </main>
      </div>
    );
  }

  const selectedWorkspace = workspaces.find((w) => w.name === selected) ?? null;

  return (
    <div className="app-shell">
      <aside className="rail">
        <div className="brand-mark">LO</div>
        <div className="brand-copy">
          <p className="eyebrow">LotusOS</p>
          <h1>Shell</h1>
          <p className="caption">workspace runtime</p>
        </div>

        <nav className="nav">
          <button
            type="button"
            className={view === "add" ? "nav-item active" : "nav-item"}
            onClick={() => setView("add")}
          >
            <span>+ Add workspace</span>
          </button>
          {view === "list"
            ? workspaces.map((w) => (
                <button
                  key={w.key}
                  type="button"
                  className={w.name === selected ? "nav-item active" : "nav-item"}
                  onClick={() => setSelected(w.name)}
                >
                  <span>{w.name}</span>
                  <span className={stateClass(w.state)}>
                    {STATE_LABELS[w.state] ?? w.state.toUpperCase()}
                  </span>
                </button>
              ))
            : null}
        </nav>

        {notice ? <p className="caption error-text">{notice}</p> : null}
      </aside>

      <main className="workspace">
        {view === "add" ? (
          <AddWorkspaceFlow
            onAdded={(name) => {
              setView("list");
              setSelected(name);
              refreshList();
            }}
          />
        ) : selectedWorkspace ? (
          <WorkspaceDetail
            workspace={selectedWorkspace}
            status={status}
            onAction={act}
            busy={busy}
          />
        ) : (
          <header className="hero">
            <p className="eyebrow">No workspaces yet</p>
            <h2>Add your first workspace to begin.</h2>
            <p className="description">
              A workspace is any project folder containing a versioned lotus.toml.
            </p>
          </header>
        )}
      </main>
    </div>
  );
}

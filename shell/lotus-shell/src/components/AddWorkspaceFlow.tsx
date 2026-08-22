import { useState } from "react";
import { api } from "../api";
import type { TrustReview } from "../types";

type Props = {
  onAdded: (name: string) => void;
};

export function AddWorkspaceFlow({ onAdded }: Props) {
  const [path, setPath] = useState("");
  const [reviewData, setReviewData] = useState<TrustReview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const review = async () => {
    setError(null);
    setReviewData(null);
    if (!path.trim()) return;
    setBusy(true);
    try {
      setReviewData(await api.reviewWorkspace(path.trim()));
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const add = async (trust: boolean) => {
    if (!reviewData) return;
    setBusy(true);
    setError(null);
    try {
      await api.addWorkspace(path.trim(), trust);
      onAdded(reviewData.name);
      setPath("");
      setReviewData(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="detail">
      <header className="hero">
        <p className="eyebrow">Add workspace</p>
        <h2>Register a project folder with a lotus.toml</h2>
        <p className="description">
          Review the manifest first. Commands only run after an explicit trust decision,
          and any later manifest change requires re-approval.
        </p>
        <div className="add-row">
          <input
            className="text-input"
            placeholder="C:\projects\my-app"
            value={path}
            onChange={(e) => setPath(e.target.value)}
          />
          <button type="button" disabled={busy || !path.trim()} onClick={review}>
            Inspect
          </button>
        </div>
        {error ? <p className="error-text">{error}</p> : null}
      </header>

      {reviewData ? (
        <article className="panel">
          <div className="panel-heading">
            <h3>{reviewData.name}</h3>
            <span className="panel-copy mono-small">{reviewData.manifest_hash.slice(0, 16)}…</span>
          </div>
          {reviewData.description ? <p>{reviewData.description}</p> : null}
          <table className="proc-table">
            <thead><tr><th>Process</th><th>Command</th><th>Ports</th></tr></thead>
            <tbody>
              {reviewData.processes.map((p) => (
                <tr key={p.name}>
                  <td>{p.name}</td>
                  <td className="mono-small">{p.command} {p.args.join(" ")}</td>
                  <td>{p.ports.length > 0 ? p.ports.join(", ") : "-"}</td>
                </tr>
              ))}
            </tbody>
          </table>
          {reviewData.env_required.length > 0 ? (
            <p className="panel-note">Requires env vars (names only): {reviewData.env_required.join(", ")}</p>
          ) : null}
          <div className="action-row">
            <button
              type="button"
              className="accent"
              disabled={busy}
              onClick={() => add(true)}
            >
              Add &amp; trust this workspace
            </button>
            <button type="button" disabled={busy} onClick={() => add(false)}>
              Add without trust
            </button>
          </div>
        </article>
      ) : null}
    </section>
  );
}

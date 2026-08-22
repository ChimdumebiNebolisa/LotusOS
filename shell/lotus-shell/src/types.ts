export type WorkspaceListEntry = {
  key: string;
  name: string;
  root: string;
  added_at: string;
  state: string;
  supervisor_alive: boolean;
  trusted: boolean;
  manifest_drift: boolean;
};

export type ProcessStatus = {
  name: string;
  state: string;
  pid?: number | null;
  identity_token?: number;
  healthy?: boolean | null;
  restarts: number;
  exit_code?: number | null;
  detail?: string | null;
};

export type PortConflict = {
  port: number;
  expected_owner: string;
  owner_pid?: number | null;
  owner_name?: string | null;
  owned_by_workspace: boolean;
  remediation: string;
};

export type StatusReport = {
  key: string;
  name: string;
  root: string;
  manifest_hash: string;
  state: string;
  started_at_ms?: number | null;
  updated_at_ms: number;
  processes: ProcessStatus[];
  port_conflicts: PortConflict[];
  last_error?: string | null;
};

export type FindingStatus =
  | "ok"
  | "missing"
  | "invalid"
  | "unverified"
  | "conflict";

export type Finding = {
  check: string;
  subject: string;
  status: FindingStatus;
  message: string;
};

export type LedgerEvent = {
  seq: number;
  ts_ms: number;
  kind: string;
  process?: string | null;
};

export type Checkpoint = {
  id: string;
  created_at_ms: number;
  created_at: string;
  workspace_name: string;
  root: string;
  manifest_hash: string;
  git_branch?: string | null;
  git_commit?: string | null;
  git_dirty?: boolean | null;
  processes: string[];
  ports: number[];
  last_state?: string | null;
  note?: string | null;
};

export type Drift = { kind: string; expected: string; found: string };

export type RestorePreview = { checkpoint: Checkpoint; drift: Drift[] };

export type ProcessReview = {
  name: string;
  command: string;
  args: string[];
  ports: number[];
};

export type TrustReview = {
  key: string;
  name: string;
  description?: string | null;
  root: string;
  processes: ProcessReview[];
  env_required: string[];
  manifest_hash: string;
};

export const STATE_LABELS: Record<string, string> = {
  off: "OFF",
  starting: "STARTING",
  healthy: "HEALTHY",
  degraded: "DEGRADED",
  failed: "FAILED",
  stopping: "STOPPING",
};

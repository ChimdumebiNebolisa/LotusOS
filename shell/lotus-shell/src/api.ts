import { invoke } from "@tauri-apps/api/core";
import type {
  Checkpoint,
  Finding,
  LedgerEvent,
  RestorePreview,
  StatusReport,
  TrustReview,
  WorkspaceListEntry,
} from "./types";

export const api = {
  listWorkspaces: () => invoke<WorkspaceListEntry[]>("list_workspaces"),
  reviewWorkspace: (path: string) =>
    invoke<TrustReview>("review_workspace", { path }),
  addWorkspace: (path: string, trust: boolean) =>
    invoke<string>("add_workspace", { path, trust }),
  grantTrust: (selector: string) => invoke<void>("grant_trust", { selector }),
  removeWorkspace: (selector: string) =>
    invoke<void>("remove_workspace", { selector }),
  startWorkspace: (selector: string) =>
    invoke<void>("start_workspace", { selector }),
  stopWorkspace: (selector: string) =>
    invoke<string[]>("stop_workspace", { selector }),
  restartWorkspace: (selector: string) =>
    invoke<void>("restart_workspace", { selector }),
  status: (selector: string) => invoke<StatusReport>("workspace_status", { selector }),
  doctor: (selector: string) => invoke<Finding[]>("doctor_workspace", { selector }),
  events: (selector: string, limit = 40) =>
    invoke<LedgerEvent[]>("workspace_events", { selector, limit }),
  logs: (selector: string, process?: string, lines = 60) =>
    invoke<string[]>("workspace_logs", { selector, process, lines }),
  createCheckpoint: (selector: string, note?: string) =>
    invoke<Checkpoint>("create_checkpoint", { selector, note }),
  checkpoints: (selector: string) =>
    invoke<Checkpoint[]>("list_checkpoints", { selector }),
  restorePreview: (selector: string, checkpointId: string) =>
    invoke<RestorePreview>("restore_preview", { selector, checkpointId }),
  restore: (selector: string, checkpointId: string) =>
    invoke<DriftLike[]>("restore_workspace", { selector, checkpointId }),
};

type DriftLike = { kind: string; expected: string; found: string };

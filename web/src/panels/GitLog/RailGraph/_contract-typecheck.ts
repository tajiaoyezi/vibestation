// MVP-12 Phase A · IPC contract type assertions
// This file is NOT a runtime component — it exists solely to anchor the ts-rs
// binding shapes so that `pnpm typecheck` fails immediately if any Rust field
// is renamed without updating the frontend (H2 regression detection, spec §G.4).
//
// Keep this file in sync with crates/core/src/rail_graph_events.rs.

import type {
  RailGraphBranchChangedPayload,
  RailGraphPerfSample,
  RailGraphRebaseStatePayload,
  RailGraphViewportSyncPayload,
} from "../../../bindings";

// Structural checks — TypeScript will error if a field is renamed or removed.
type _ViewportSyncCheck = {
  workspaceId: RailGraphViewportSyncPayload["workspaceId"];
  scrollTop: RailGraphViewportSyncPayload["scrollTop"];
  rowHeight: RailGraphViewportSyncPayload["rowHeight"];
  viewportStart: RailGraphViewportSyncPayload["viewportStart"];
  viewportEnd: RailGraphViewportSyncPayload["viewportEnd"];
};

type _BranchChangedCheck = {
  workspaceId: RailGraphBranchChangedPayload["workspaceId"];
  headOid: RailGraphBranchChangedPayload["headOid"];
  refsHash: RailGraphBranchChangedPayload["refsHash"]; // H2 anchor — renames this breaks typecheck
  branchCount: RailGraphBranchChangedPayload["branchCount"];
};

type _RebaseStateCheck = {
  workspaceId: RailGraphRebaseStatePayload["workspaceId"];
  state: RailGraphRebaseStatePayload["state"];
};

type _PerfSampleCheck = {
  workspaceId: RailGraphPerfSample["workspaceId"];
  phase: RailGraphPerfSample["phase"];
  durationMs: RailGraphPerfSample["durationMs"]; // #[ts(type = "number")] verified
  commitCount: RailGraphPerfSample["commitCount"];
  branchCount: RailGraphPerfSample["branchCount"];
};

// Ensure type shapes are used (prevent noUnusedLocals warning)
export type ContractChecks =
  | _ViewportSyncCheck
  | _BranchChangedCheck
  | _RebaseStateCheck
  | _PerfSampleCheck;

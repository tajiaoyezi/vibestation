// 临时契约 · 权威源 = MVP-18 spec §K · A1 ts-rs 落地后由 Wave 2 替换为 import @/bindings/*
// TODO(Wave-2): replace all exports in this file with generated `@/bindings/*` imports.
// 勿扩展字段，保持与 spec §K.3/§K.4/§K.6 对齐。

export type PaneLinkKind = "failureFeedback";

export type PaneLinkStatus = "enabled" | "disabled" | "stale" | "removed";

export type PaneFailureFallbackMode = "structured" | "rawText";

export type ParsedIssueSeverity = "error" | "warning" | "info";

export interface ParsedIssue {
  severity: ParsedIssueSeverity;
  file: string | null;
  line: number | null;
  column: number | null;
  message: string;
}

export interface PaneLink {
  workspaceId: string;
  linkId: string;
  parentPaneId: string;
  childPaneId: string;
  linkKind: PaneLinkKind;
  status: PaneLinkStatus;
  updatedAt: number;
}

export interface PaneLinkedEvent {
  workspaceId: string;
  linkId: string;
  parentPaneId: string;
  childPaneId: string;
  linkKind: PaneLinkKind;
  status: PaneLinkStatus;
  updatedAt: number;
}

export interface PaneTriggerEvent {
  workspaceId: string;
  childPaneId: string;
  commandRunId: string;
  reason: string;
  exitCode: number | null;
  command: string;
  occurredAt: number;
}

export interface PaneBuildFailedEvent {
  workspaceId: string;
  linkId: string;
  parentPaneId: string;
  childPaneId: string;
  commandRunId: string;
  exitCode: number | null;
  rawExcerpt: string;
  parsedIssues: ParsedIssue[];
  parserConfidence: number;
  fallbackMode: PaneFailureFallbackMode;
  occurredAt: number;
}

import { invoke } from "@tauri-apps/api/core";

export interface RuntimeConfig {
  scenario?: string | null;
  outputDir?: string | null;
  reportDir?: string | null;
  runLabel?: string | null;
  closeOnComplete: boolean;
}

export interface SpawnSessionRequest {
  label: string;
  command: string;
  cols: number;
  rows: number;
  queueCapacity?: number;
}

export interface WriteSessionRequest {
  sessionId: string;
  data: string;
}

export interface ResizeSessionRequest {
  sessionId: string;
  cols: number;
  rows: number;
}

export interface DrainSessionRequest {
  sessionId: string;
  maxChunks?: number;
  maxBytes?: number;
}

export interface SessionSummary {
  id: string;
  label: string;
  command: string;
  queueDepth: number;
  queuedBytes: number;
  avgQueueDepth: number;
  maxQueueDepth: number;
  totalReadBytes: number;
  totalDrainedBytes: number;
  droppedChunks: number;
  droppedBytes: number;
  exitStatus?: string | null;
  createdAtMs: number;
}

export interface DrainResponse {
  chunks: string[];
  queueDepth: number;
  queuedBytes: number;
  avgQueueDepth: number;
  maxQueueDepth: number;
  totalReadBytes: number;
  totalDrainedBytes: number;
  droppedChunks: number;
  droppedBytes: number;
  exitStatus?: string | null;
}

export interface ProcessStats {
  pid: number;
  rssKb: number;
  fdCount: number;
  sessionCount: number;
  readerThreadAlive: boolean;
}

export interface ArtifactWriteRequest {
  path: string;
  contents: string;
}

export interface ArtifactReadRequest {
  path: string;
}

export const runtimeConfig = (): Promise<RuntimeConfig> => invoke("runtime_config");
export const spawnSession = (request: SpawnSessionRequest): Promise<SessionSummary> =>
  invoke("spawn_session", { request });
export const writeSession = (request: WriteSessionRequest): Promise<void> =>
  invoke("write_session", { request });
export const resizeSession = (request: ResizeSessionRequest): Promise<void> =>
  invoke("resize_session", { request });
export const drainSession = (request: DrainSessionRequest): Promise<DrainResponse> =>
  invoke("drain_session", { request });
export const closeSession = (sessionId: string): Promise<void> =>
  invoke("close_session", { sessionId });
export const closeAllSessions = (): Promise<void> => invoke("close_all_sessions");
export const sessionSnapshot = (sessionId: string): Promise<SessionSummary> =>
  invoke("session_snapshot", { sessionId });
export const managerSnapshot = (): Promise<SessionSummary[]> => invoke("manager_snapshot");
export const writeArtifact = (request: ArtifactWriteRequest): Promise<void> =>
  invoke("write_artifact", { request });
export const readArtifact = (request: ArtifactReadRequest): Promise<string> =>
  invoke("read_artifact", { request });
export const sampleProcessStats = (): Promise<ProcessStats> => invoke("sample_process_stats");
export const exitApp = (): Promise<void> => invoke("exit_app");

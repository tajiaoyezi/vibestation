// TODO(MVP-20 M2 merged): 替换为 import from "@/bindings/RollbackPreview" · 删除本地定义
export interface RollbackPreview {
  session_id: string;
  commits: RollbackCommitEntry[];
  total_files_changed: number;
  total_insertions: number;
  total_deletions: number;
  has_low_confidence: boolean;
}

// TODO(MVP-20 M2 merged): 替换为 import from "@/bindings/RollbackCommitEntry" · 删除本地定义
export interface RollbackCommitEntry {
  sha: string;
  message: string;
  author: string;
  timestamp: number;
  confidence: number;
  include: boolean;
  files_changed: number;
}

// TODO(MVP-20 M2 merged): 替换为 import from "@/bindings/RollbackProgress" · 删除本地定义
export interface RollbackProgress {
  done: number;
  total: number;
  current_sha: string;
  status: string;
}

// TODO(MVP-20 M2 merged): 替换为 import from "@/bindings/RollbackAbortResult" · 删除本地定义
export interface RollbackAbortResult {
  success: boolean;
  head_sha: string;
  error: string | null;
}

// TODO(MVP-20 M2 merged): 替换为 import from "@/bindings/RollbackStatus" · 删除本地定义
export interface RollbackStatus {
  session_id: string;
  status: "idle" | "in_progress" | "conflict_paused" | "completed" | "aborted";
  current_idx: number;
  total: number;
  current_sha: string | null;
}

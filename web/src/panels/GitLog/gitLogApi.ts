import { invoke } from "@tauri-apps/api/core";
import type {
  GitLogQueryRequest,
  GitLogQueryResponse,
  CommitDetail,
} from "../../bindings";

export async function queryLog(
  req: GitLogQueryRequest,
): Promise<GitLogQueryResponse> {
  return invoke<GitLogQueryResponse>("git_log_query", { req });
}

export async function fetchDetail(
  workspaceId: string,
  sha: string,
): Promise<CommitDetail> {
  return invoke<CommitDetail>("git_log_commit_detail", {
    workspaceId,
    sha,
  });
}

export async function clearCache(): Promise<void> {
  return invoke("git_log_cache_clear");
}

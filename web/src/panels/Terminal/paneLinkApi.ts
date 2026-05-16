import { invoke } from "@tauri-apps/api/core";
import type {
  PaneFailurePreviewRequest,
  PaneFailurePreviewResult,
  PaneLinkRequest,
  PaneLinkResult,
  PaneLinkSetEnabledRequest,
  PaneLinksListRequest,
  PaneLinksListResult,
  PaneUnlinkRequest,
  PaneUnlinkResult,
} from "../../bindings";

export async function linkPanes(req: PaneLinkRequest): Promise<PaneLinkResult> {
  return invoke<PaneLinkResult>("pane_link", { req });
}

export async function unlinkPane(
  req: PaneUnlinkRequest,
): Promise<PaneUnlinkResult> {
  return invoke<PaneUnlinkResult>("pane_unlink", { req });
}

export async function listPaneLinks(
  req: PaneLinksListRequest,
): Promise<PaneLinksListResult> {
  return invoke<PaneLinksListResult>("pane_links_list", { req });
}

export async function setPaneLinkEnabled(
  req: PaneLinkSetEnabledRequest,
): Promise<PaneLinkResult> {
  return invoke<PaneLinkResult>("pane_links_set_enabled", { req });
}

export async function previewFailurePrompt(
  req: PaneFailurePreviewRequest,
): Promise<PaneFailurePreviewResult> {
  return invoke<PaneFailurePreviewResult>("pane_failure_preview_prompt", {
    req,
  });
}

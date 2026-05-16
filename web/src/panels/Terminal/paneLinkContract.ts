// Wave-2 bridge · re-exports from generated bindings + store local types.
// Phase C merged 后此文件可删除 · 所有消费者应直接 import bindings 和 stores/paneLinks。
export type {
  PaneLink,
  PaneLinkKind,
  PaneLinkStatus,
  PaneLinkedEvent,
  PaneTriggerEvent,
  PaneLinkError,
  PaneLinkErrorEvent,
} from "../../bindings";

export type { PaneBuildFailedEvent } from "../../stores/paneLinks";

// MOCK · 等 MVP-17 Phase A PR merge 后删 · 改 import "../EnvPreview"

import type { EnvEntry } from "./EnvEntry";

export interface EnvPreview {
  /** 可见 env 条目（白名单通过 + 未截断） */
  visibleEntries: EnvEntry[];
  /** 被过滤掉的条目数（黑名单命中） */
  filteredCount: number;
}

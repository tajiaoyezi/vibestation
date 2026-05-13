// MOCK · 等 MVP-17 Phase A PR merge 后删 · 改 import "../EnvEntry"

export interface EnvEntry {
  /** 环境变量名 */
  key: string;
  /** 截断后的值（最长 40 字符） */
  valueTruncated: string;
  /** 是否已脱敏（黑名单命中 · 显示为 ***） */
  isSensitiveRedacted: boolean;
}

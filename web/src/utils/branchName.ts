const FORBIDDEN_CHARS = /[\s\x00-\x1f\x7f~^:?*\[\\]/;
const FORBIDDEN_PATTERNS = [/^[./]/, /\.\.+/, /@\{/, /\.git/, /\.lock$/, /\/$/];

export interface BranchNameValidation {
  valid: boolean;
  reason?: string;
}

export function validateBranchName(name: string): BranchNameValidation {
  const trimmed = name.trim();

  if (!trimmed) {
    return { valid: false, reason: "分支名不能为空" };
  }

  if (trimmed !== name) {
    return { valid: false, reason: "分支名首尾不能包含空格" };
  }

  if (FORBIDDEN_CHARS.test(trimmed)) {
    return {
      valid: false,
      reason: "含非法字符（空格 / 控制字符 / ~^:?*[\\）",
    };
  }

  for (const pattern of FORBIDDEN_PATTERNS) {
    if (pattern.test(trimmed)) {
      return {
        valid: false,
        reason: "非法格式（起始 . 或 / · 含 .. / @{ / .git · 结尾 .lock 或 /）",
      };
    }
  }

  return { valid: true };
}

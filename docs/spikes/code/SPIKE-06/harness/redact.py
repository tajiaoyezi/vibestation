#!/usr/bin/env python3
"""
SPIKE-06 redaction · 按 spec §A.5.2 替换敏感值 · 保留结构

脱敏清单（spec §A.5.2）:
- auth token / API key / JWT / session cookie / Bearer
- 本地路径 /Users/<name> · /home/<name>
- git remote URL
- 邮箱 / 电话 / 身份证号 等 PII
- git config user.name / user.email

脱敏原则（spec §A.5.4）:
- 结构保留（JWT 保持 3-part 占位 · 路径保持 /Users/USER 占位）
- 值丢失（不留真实 token / email / path）

Usage: ./redact.py --input <raw-path> --output <redacted-path>

Exit code:
- 0: OK · 脱敏完成（不论是否 match）
- 1: 输入不存在
- 2: 写入失败
"""

import sys
import re
import argparse
from pathlib import Path

# ============================================================
# 脱敏规则 · 按风险级别排序（高风险先匹配 · 避免被低规则吞）
# ============================================================

PATTERNS = [
    # --- 认证凭据（高风险）---

    # Anthropic API key (sk-ant- 前缀)
    (r'\bsk-ant-[A-Za-z0-9_\-]{20,}', 'sk-ant-REDACTED_ANTHROPIC_KEY'),

    # OpenAI API key (sk- 前缀 + 40+ 字符)
    (r'\bsk-[A-Za-z0-9]{40,}', 'sk-REDACTED_OPENAI_KEY'),

    # JWT (3 base64 段)
    (r'\beyJ[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+\b',
     'eyJ...REDACTED_JWT_HEADER....REDACTED_JWT_PAYLOAD....REDACTED_JWT_SIG'),

    # Bearer / Authorization header
    (r'(?i)(Bearer|Authorization:)\s+[A-Za-z0-9_\-\.+/=]{20,}',
     r'\1 REDACTED_AUTH_TOKEN'),

    # github token (ghp_ / gho_ / ghs_ / github_pat_)
    (r'\b(ghp_|gho_|ghs_|ghr_|github_pat_)[A-Za-z0-9_]{20,}',
     r'\1REDACTED_GITHUB_TOKEN'),

    # --- 本地路径 ---

    # macOS
    (r'/Users/[a-zA-Z0-9_\-\.]+', '/Users/USER'),

    # Linux
    (r'/home/[a-zA-Z0-9_\-\.]+', '/home/USER'),

    # --- Git remote ---

    # HTTPS github
    (r'https?://(?:[a-zA-Z0-9_\-]+@)?github\.com/([a-zA-Z0-9_\-]+)/([a-zA-Z0-9_\-\.]+?)(\.git)?(?=[\s"\']|$)',
     r'https://github.com/EXAMPLE/REPO\3'),

    # SSH github
    (r'git@github\.com:[a-zA-Z0-9_\-]+/[a-zA-Z0-9_\-\.]+?\.git',
     'git@github.com:EXAMPLE/REPO.git'),

    # --- PII ---

    # Email
    (r'[a-zA-Z0-9_.+\-]+@[a-zA-Z0-9\-]+\.[a-zA-Z0-9\-\.]+',
     'user@EXAMPLE.COM'),

    # 中国手机号
    (r'\b1[3-9]\d{9}\b', '1XXXXXXXXXX'),

    # 身份证号（18 位）
    (r'\b[1-9]\d{5}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx]\b',
     'REDACTED_ID_18'),
]


def redact(text: str) -> tuple[str, list[tuple[str, int]]]:
    """返回 (脱敏后文本, [(规则预览, 命中次数), ...])"""
    applied = []
    for pattern, replacement in PATTERNS:
        new_text, n = re.subn(pattern, replacement, text)
        if n > 0:
            preview = pattern[:50] + ('...' if len(pattern) > 50 else '')
            applied.append((preview, n))
        text = new_text
    return text, applied


def main() -> int:
    parser = argparse.ArgumentParser(description='SPIKE-06 sample redactor')
    parser.add_argument('--input', required=True, help='raw path (usually ~/.vibestation-spike-raw/...)')
    parser.add_argument('--output', required=True, help='redacted path (docs/spikes/raw/SPIKE-06/*.txt)')
    args = parser.parse_args()

    in_path = Path(args.input).expanduser()
    out_path = Path(args.output).expanduser()

    if not in_path.exists():
        print(f'❌ input not found: {in_path}', file=sys.stderr)
        return 1

    try:
        raw = in_path.read_text(encoding='utf-8', errors='replace')
    except Exception as e:
        print(f'❌ read failed: {e}', file=sys.stderr)
        return 1

    redacted, applied = redact(raw)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        out_path.write_text(redacted, encoding='utf-8')
    except Exception as e:
        print(f'❌ write failed: {e}', file=sys.stderr)
        return 2

    print(f'✅ redacted: {in_path} → {out_path}')
    print(f'   size: {len(raw)} → {len(redacted)} bytes')
    if applied:
        print('   patterns matched:')
        for preview, n in applied:
            print(f'     - {preview}  ×{n}')
    else:
        print('   (no patterns matched · source was zero-sensitive · OK for smoke)')

    return 0


if __name__ == '__main__':
    sys.exit(main())

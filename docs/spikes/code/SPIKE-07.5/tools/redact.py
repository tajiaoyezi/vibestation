#!/usr/bin/env python3
"""SPIKE-07.5 corpus 脱敏器（沿用 SPIKE-06 纪律 · §C.1）。

输入 /tmp/spike075-raw/*.structured.jsonl（未脱敏）
输出 docs/spikes/raw/SPIKE-07.5/corpus/{name}.structured.jsonl（脱敏）
     + {name}.redaction.json sidecar（记录脱敏字段计数 · 对齐 SPIKE-06）

脱敏规则（保协议结构 · 仅替换敏感值为占位）：
- UUID 值（session_id / uuid / thread_id / hook_id / item id 等）→ <REDACTED_UUID>
- 本机绝对路径 /Users/<user>/... 与 /private/tmp/... → <REDACTED_PATH>
- API base url override 痕迹 / api key 形态（sk-...）→ <REDACTED_SECRET>
- 不改事件结构 / type / 顺序（parser 验证需要协议结构完整）
"""
import json
import re
import sys
from pathlib import Path

UUID_RE = re.compile(
    r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b"
)
PATH_RE = re.compile(r"(/Users/[^/\s\"]+|/private/tmp|/home/[^/\s\"]+)[^\s\"]*")
SECRET_RE = re.compile(r"\bsk-[A-Za-z0-9_-]{6,}\b")


def redact_text(s: str, counts: dict) -> str:
    def _u(m):
        counts["uuid"] = counts.get("uuid", 0) + 1
        return "<REDACTED_UUID>"

    def _p(m):
        counts["path"] = counts.get("path", 0) + 1
        return "<REDACTED_PATH>"

    def _s(m):
        counts["secret"] = counts.get("secret", 0) + 1
        return "<REDACTED_SECRET>"

    s = SECRET_RE.sub(_s, s)
    s = UUID_RE.sub(_u, s)
    s = PATH_RE.sub(_p, s)
    return s


def main():
    src = Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/spike075-raw")
    dst = Path(sys.argv[2])
    dst.mkdir(parents=True, exist_ok=True)
    files = sorted(src.glob("*.structured.jsonl"))
    total = 0
    for f in files:
        counts: dict = {}
        out_lines = []
        for ln in f.read_text().splitlines():
            if not ln.strip():
                continue
            # 行级文本脱敏（保 JSON 结构 · 值内 UUID/path/secret 替换）
            try:
                json.loads(ln)  # 确认是合法 JSON（不合法则原样 · loader 容错）
                out_lines.append(redact_text(ln, counts))
            except Exception:
                out_lines.append(redact_text(ln, counts))
        stem = f.name[: -len(".structured.jsonl")]
        (dst / f.name).write_text("\n".join(out_lines) + "\n")
        sidecar = {
            "sample": stem,
            "redacted_fields": [{"kind": k, "count": v} for k, v in sorted(counts.items())],
            "source_lines": len(out_lines),
            "redaction_tool": "SPIKE-07.5/tools/redact.py",
        }
        (dst / f"{stem}.redaction.json").write_text(
            json.dumps(sidecar, ensure_ascii=False, indent=2) + "\n"
        )
        total += 1
        print(f"  {stem}: lines={len(out_lines)} redacted={counts}")
    print(f"DONE · {total} samples → {dst}")


if __name__ == "__main__":
    main()

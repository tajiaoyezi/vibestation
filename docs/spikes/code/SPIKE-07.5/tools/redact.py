#!/usr/bin/env python3
"""SPIKE-07.5 corpus 脱敏器 v2 · 结构保留型（沿用 SPIKE-06 纪律 · §C.1）。

输入 /tmp/spike075-raw/*.structured.jsonl（未脱敏 ground truth · 一行一 JSON 事件）
输出 docs/spikes/raw/SPIKE-07.5/corpus/{name}.structured.jsonl（脱敏 · 仍一行一事件）
     + {name}.redaction.json sidecar（脱敏字段计数 · 对齐 SPIKE-06）

## v1 → v2 根因修复（2026-05-16 · decision-grade 证据完整性）

v1（regex 文本替换）有 JSON 转义破坏 bug：`PATH_RE` 尾随 `[^\\s\\"]*`
吃掉 claude `hook_response.output`（双重转义嵌套 JSON 字符串）里路径前的
转义反斜杠 `\\`，把 `<path>\\",\\"cwd\\"` 改成 `<REDACTED_PATH>",\\"cwd\\"`
→ 该物理行变非法 JSON。实测 184/936 行被污染（误记为"多行续行"）。
raw `/tmp/spike075-raw` 实测 36/36 文件 936/936 行**严格一行一合法 JSON ·
零多行事件**——污染 100% 由 v1 文本替换引入，非协议现象。

v2 修复：**先 `json.loads` 解析整事件 → 递归只对字符串叶子做脱敏 →
`json.dumps` 重序列化**。脱敏作用于已解析的字符串"值"，json 重序列化
负责转义 → 嵌套转义结构不可能被破坏。协议结构 / type / 顺序 / 键名
100% 保留（仅敏感叶子值替换为占位 · 不可逆 by design）。

不脱敏键名（协议字段 type/subtype/session_id/... 非敏感 · parser 验证需要）。
仅脱敏字符串值内的 UUID / 本机绝对路径 / api key 形态。
"""

import json
import re
import sys
from pathlib import Path

UUID_RE = re.compile(
    r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b"
)
# 路径：到空白 / 引号 / 反斜杠 / 逗号 即停（嵌套转义场景安全 · 不再吞转义符）。
PATH_RE = re.compile(r"(?:/Users/|/private/tmp|/home/)[^\s\"\\,]*")
SECRET_RE = re.compile(r"\bsk-[A-Za-z0-9_-]{6,}\b")


def redact_str(s: str, counts: dict) -> str:
    """对单个字符串"值"脱敏（json.dumps 负责重新转义 · 不碰 JSON 语法）。"""

    def _s(m):
        counts["secret"] = counts.get("secret", 0) + 1
        return "<REDACTED_SECRET>"

    def _u(m):
        counts["uuid"] = counts.get("uuid", 0) + 1
        return "<REDACTED_UUID>"

    def _p(m):
        counts["path"] = counts.get("path", 0) + 1
        return "<REDACTED_PATH>"

    s = SECRET_RE.sub(_s, s)
    s = UUID_RE.sub(_u, s)
    s = PATH_RE.sub(_p, s)
    return s


def walk(node, counts):
    """递归：仅脱敏字符串叶子值；键名 / 数字 / bool / null 原样。"""
    if isinstance(node, str):
        return redact_str(node, counts)
    if isinstance(node, list):
        return [walk(x, counts) for x in node]
    if isinstance(node, dict):
        # 键名不脱敏（协议字段名）· 仅递归值
        return {k: walk(v, counts) for k, v in node.items()}
    return node


def main():
    src = Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/spike075-raw")
    dst = Path(sys.argv[2])
    dst.mkdir(parents=True, exist_ok=True)
    files = sorted(src.glob("*.structured.jsonl"))
    total = 0
    grand_bad = 0
    for f in files:
        counts: dict = {}
        out_lines = []
        bad = 0
        for ln in f.read_text(encoding="utf-8", errors="replace").splitlines():
            s = ln.strip()
            if not s:
                continue
            try:
                obj = json.loads(s)
            except Exception:
                # raw 实测 0 此分支 · 防御：真截断尾原样文本脱敏（密钥不泄）+ 计数
                bad += 1
                out_lines.append(redact_str(s, counts))
                continue
            red = walk(obj, counts)
            # 紧凑单行 · ensure_ascii=False 保中文可读 · json 负责转义
            out_lines.append(json.dumps(red, ensure_ascii=False, separators=(",", ":")))
        stem = f.name[: -len(".structured.jsonl")]
        (dst / f.name).write_text("\n".join(out_lines) + "\n", encoding="utf-8")
        sidecar = {
            "sample": stem,
            "redacted_fields": [
                {"kind": k, "count": v} for k, v in sorted(counts.items())
            ],
            "source_lines": len(out_lines),
            "non_json_passthrough": bad,
            "redaction_tool": "SPIKE-07.5/tools/redact.py v2 (structure-preserving)",
        }
        (dst / f"{stem}.redaction.json").write_text(
            json.dumps(sidecar, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        total += 1
        grand_bad += bad
        print(f"  {stem}: lines={len(out_lines)} bad_passthrough={bad} redacted={counts}")
    print(f"DONE · {total} samples → {dst} · 总 non-JSON passthrough={grand_bad}")


if __name__ == "__main__":
    main()

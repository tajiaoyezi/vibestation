#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "$0")" && pwd)/common.sh"

RAW_DIR="${SPIKE08_RAW_DIR:-$RAW_DIR_DEFAULT}"
ensure_raw_dir "$RAW_DIR"

cd "$ROOT_DIR"

TMP_DIR="$(mktemp -d)"
cp src-tauri/src/contract.rs "$TMP_DIR/contract.rs"
cp src-tauri/src/store.rs "$TMP_DIR/store.rs"

restore() {
  cp "$TMP_DIR/contract.rs" src-tauri/src/contract.rs
  cp "$TMP_DIR/store.rs" src-tauri/src/store.rs
  cargo build --manifest-path src-tauri/Cargo.toml >/dev/null
}

trap restore EXIT INT TERM

python3 - <<'PY'
from pathlib import Path

contract = Path("src-tauri/src/contract.rs")
store = Path("src-tauri/src/store.rs")

contract_text = contract.read_text()
contract_text = contract_text.replace("    pub id: String,\n", "    pub workspace_id: String,\n")
contract.write_text(contract_text)

store_text = store.read_text()
store_text = store_text.replace("            id: format!(\"workspace-{next_id:04}\"),\n", "            workspace_id: format!(\"workspace-{next_id:04}\"),\n")
store_text = store_text.replace("workspace.id != request.workspace_id", "workspace.workspace_id != request.workspace_id")
store.write_text(store_text)
PY

cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tee "$RAW_DIR/h2-contract-regression-build.log"

set +e
pnpm typecheck 2>&1 | tee "$RAW_DIR/h2-contract-regression.log"
STATUS=${PIPESTATUS[0]}
set -e

if [[ "$STATUS" -eq 0 ]]; then
  echo "Expected pnpm typecheck to fail after Rust rename regression" >&2
  exit 1
fi

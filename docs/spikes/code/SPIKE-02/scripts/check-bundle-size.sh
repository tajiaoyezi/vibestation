#!/usr/bin/env bash
# SPIKE-02 · macOS / Ubuntu bundle size 校验
# 用法：./scripts/check-bundle-size.sh
#
# 验收：macOS dmg < 30MB · Ubuntu AppImage/deb < 40MB
# 输出：每个产物的大小 + PASS/FAIL 判定

set -uo pipefail

PROJ_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUNDLE_ROOT="${PROJ_ROOT}/src-tauri/target/release/bundle"

if [[ ! -d "${BUNDLE_ROOT}" ]]; then
  echo "ERROR: Bundle dir not found: ${BUNDLE_ROOT}"
  echo "请先跑：pnpm tauri build"
  exit 1
fi

echo "Bundle size check · SPIKE-02"
echo "Bundle root: ${BUNDLE_ROOT}"
echo "---"

MAC_DMG_LIMIT_MB=30
UBUNTU_LIMIT_MB=40

OS_TYPE="$(uname -s)"
FAIL=0

# macOS 产物
if [[ -d "${BUNDLE_ROOT}/macos" ]]; then
  for app in "${BUNDLE_ROOT}"/macos/*.app; do
    [[ -d "${app}" ]] || continue
    SIZE_MB=$(du -sm "${app}" | cut -f1)
    echo "macOS .app: ${app##*/} = ${SIZE_MB}MB"
  done
fi

if [[ -d "${BUNDLE_ROOT}/dmg" ]]; then
  for dmg in "${BUNDLE_ROOT}"/dmg/*.dmg; do
    [[ -f "${dmg}" ]] || continue
    SIZE_MB=$(du -sm "${dmg}" | cut -f1)
    if [[ "${SIZE_MB}" -lt "${MAC_DMG_LIMIT_MB}" ]]; then
      echo "macOS .dmg: ${dmg##*/} = ${SIZE_MB}MB ✅ PASS (< ${MAC_DMG_LIMIT_MB}MB)"
    else
      echo "macOS .dmg: ${dmg##*/} = ${SIZE_MB}MB ❌ FAIL (>= ${MAC_DMG_LIMIT_MB}MB)"
      FAIL=1
    fi
  done
fi

# Ubuntu 产物（Phase B · 有时）
for kind in deb appimage; do
  if [[ -d "${BUNDLE_ROOT}/${kind}" ]]; then
    for f in "${BUNDLE_ROOT}/${kind}"/*; do
      [[ -f "${f}" ]] || continue
      SIZE_MB=$(du -sm "${f}" | cut -f1)
      if [[ "${SIZE_MB}" -lt "${UBUNTU_LIMIT_MB}" ]]; then
        echo "Ubuntu ${kind}: ${f##*/} = ${SIZE_MB}MB ✅ PASS (< ${UBUNTU_LIMIT_MB}MB)"
      else
        echo "Ubuntu ${kind}: ${f##*/} = ${SIZE_MB}MB ❌ FAIL (>= ${UBUNTU_LIMIT_MB}MB)"
        FAIL=1
      fi
    done
  fi
done

echo "---"
if [[ "${FAIL}" == "0" ]]; then
  echo "✅ All bundle sizes within limits"
  exit 0
else
  echo "❌ One or more bundles exceed limits"
  exit 1
fi

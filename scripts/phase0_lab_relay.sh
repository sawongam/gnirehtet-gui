#!/usr/bin/env bash
# Headless Phase 0 relay ownership lab (P0-R1/R2/R4/R5 + P0-Q1 + APK_MISSING).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
HOST="$(rustc -vV | sed -n 's/^host: //p')"
BIN="${GNIREHTET_BIN:-$ROOT/apps/desktop/src-tauri/binaries/gnirehtet-$HOST}"
APK="${GNIREHTET_APK:-$ROOT/resources/gnirehtet.apk}"
export GNIREHTET_BIN="$BIN"
export GNIREHTET_APK="$APK"
echo "Using GNIREHTET_BIN=$GNIREHTET_BIN"
echo "Using GNIREHTET_APK=$GNIREHTET_APK"
if [[ ! -x "$GNIREHTET_BIN" ]]; then
  echo "Missing sidecar. Drop v2.5.1 linux64 binary at:"
  echo "  $ROOT/apps/desktop/src-tauri/binaries/gnirehtet-$HOST"
  exit 2
fi
cargo run -p gnirehtet-controller --example phase0_lab_relay "$@"

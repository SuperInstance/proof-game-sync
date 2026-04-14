#!/usr/bin/env bash
# bench.sh — build, run, and time both modes of the game-sync proof
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT"

echo "═══════════════════════════════════════════════════════════════"
echo " game-sync-proof  |  float drift vs constraint-theory snap"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# ── build ────────────────────────────────────────────────────────────────────
echo "▶ Building (release)…"
cargo build --release --bin game_sync 2>&1
echo "  Build OK"
echo ""

BINARY="./target/release/game_sync"

# ── run ──────────────────────────────────────────────────────────────────────
echo "▶ Running simulation…"
echo ""
START=$(date +%s%N)
"$BINARY"
END=$(date +%s%N)

ELAPSED_MS=$(( (END - START) / 1_000_000 ))
echo ""
echo "─────────────────────────────────────────────────────────────"
echo "  Wall time: ${ELAPSED_MS} ms"
echo "─────────────────────────────────────────────────────────────"

# ── unit tests ───────────────────────────────────────────────────────────────
echo ""
echo "▶ Running unit tests…"
cargo test --release 2>&1
echo "  Tests OK"

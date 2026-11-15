#!/usr/bin/env bash
# What CI runs (.github/workflows/ci.yml), and what to run before a pull request: scripts/ci.sh
# Format, clippy, the whole suite, on Linux. `just lint` adds cargo-deny and the Windows type-check, which need the network
# and the Windows target; CI has neither. Needs vendor/ghostty (git submodule update --init) and Zig 0.16 (ZIG or PATH).
# Tests start their own scratch cores under a throwaway home; no real core is touched.
# Two traps: the kill/interrupt tests fail when this is started under nohup (signals arrive ignored): use setsid, or a
# terminal. And each test binary aborts itself after LBC_TEST_WATCHDOG_S seconds (a hang, not a slow test); the default
# 90 is too short on a loaded machine, so this sets 600 unless you set it.
set -euo pipefail
cd "$(dirname "$0")/.."
# A throwaway home: nothing here may read or write a real config, state or socket. cargo, rustup and zig keep their own
# directories, so the build is offline from the caches that are already there.
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export ZIG_GLOBAL_CACHE_DIR="${ZIG_GLOBAL_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/zig}"
T=$(mktemp -d "${TMPDIR:-/tmp}/lci.XXXXXX")
trap 'rm -rf "$T"' EXIT
mkdir -p "$T/home" "$T/config" "$T/data" "$T/state" "$T/cache" "$T/run"
chmod 700 "$T/run"
export HOME="$T/home" XDG_CONFIG_HOME="$T/config" XDG_DATA_HOME="$T/data" XDG_STATE_HOME="$T/state" XDG_CACHE_HOME="$T/cache" XDG_RUNTIME_DIR="$T/run"
unset TMUX TMUX_PANE
for v in $(env | grep -oE '^(LBC_|LUMBRIDGE_)[A-Za-z0-9_]*' | grep -vxE 'LBC_TEST_WATCHDOG_S' || true); do unset "$v"; done
export LBC_TEST_WATCHDOG_S="${LBC_TEST_WATCHDOG_S:-600}"
rustc --version; "${ZIG:-zig}" version
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --no-fail-fast -- --test-threads "${RUST_TEST_THREADS:-4}"

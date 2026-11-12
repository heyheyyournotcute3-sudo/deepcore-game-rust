#!/usr/bin/env bash
# Format, lint and test the whole workspace. This is the gate for every milestone.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== cargo fmt =="
cargo fmt --all -- --check

echo "== cargo clippy (warnings are errors) =="
cargo clippy --workspace --all-targets -- -D warnings

echo "== cargo test =="
cargo test --workspace

echo "== all checks passed =="

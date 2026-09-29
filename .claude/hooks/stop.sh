#!/usr/bin/env bash
# Stop: block finishing while the project fails lint or tests.
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-.}"

# Already continuing because of a previous block; don't loop forever.
[[ "$(jq -r '.stop_hook_active // false')" == "true" ]] && exit 0

fail() {
  echo "$1 failed:" >&2
  echo "$2" >&2
  exit 2
}

out=$(cargo fmt --check 2>&1) || fail "cargo fmt --check" "$out"
out=$(cargo clippy --all-targets -- -D warnings 2>&1) || fail "cargo clippy" "$out"
out=$(cargo test 2>&1) || fail "cargo test" "$out"

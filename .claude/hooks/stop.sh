#!/usr/bin/env bash
# Stop: block finishing while the project fails the specification, lint or tests.
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-.}"

# Already continuing because of a previous block; don't loop forever.
[[ "$(jq -r '.stop_hook_active // false')" == "true" ]] && exit 0

fail() {
  echo "$1 failed:" >&2
  echo "$2" >&2
  exit 2
}

# The specification is cheap to check, so it goes first. CI installs sumi; a machine without it skips these.
if command -v sumi > /dev/null; then
  out=$(sumi fmt --check 2>&1) || fail "sumi fmt --check" "$out"
  out=$(sumi verify 2>&1) || fail "sumi verify" "$out"
else
  echo "sumi is not installed; the specification was not checked." >&2
fi

# React Doctor scans the playground on this machine only (--no-score sends nothing but dependency names
# for the supply-chain check) and blocks on errors; its warnings are triaged by hand.
if command -v npx > /dev/null; then
  out=$(cd playground && npx -y react-doctor@0.9.14 . --no-score -y 2>&1) || fail "react-doctor" "$out"
else
  echo "npx is not installed; the playground was not checked by React Doctor." >&2
fi

out=$(cargo fmt --all --check 2>&1) || fail "cargo fmt --check" "$out"
out=$(cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1) || fail "cargo clippy" "$out"
out=$(cargo test --workspace --locked 2>&1) || fail "cargo test" "$out"

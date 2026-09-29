#!/usr/bin/env bash
# PostToolUse (Edit|Write): keep edited Rust files formatted.
set -euo pipefail

file=$(jq -r '.tool_input.file_path // empty')

[[ "$file" == *.rs && -f "$file" ]] || exit 0

rustfmt --edition 2024 "$file"

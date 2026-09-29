#!/usr/bin/env bash
# PostToolUse (Edit|Write): keep edited Rust files and the specification formatted.
set -euo pipefail

file=$(jq -r '.tool_input.file_path // empty')

[[ -f "$file" ]] || exit 0

case "$file" in
  *.rs)
    rustfmt --edition 2024 "$file"
    ;;
  */.spec/*.md | */.sumi.json)
    # sumi formats the whole specification, so it runs from the project root.
    command -v sumi > /dev/null || exit 0
    cd "${CLAUDE_PROJECT_DIR:-.}"
    sumi fmt
    ;;
esac

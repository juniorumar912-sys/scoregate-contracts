#!/usr/bin/env bash
set -euo pipefail

BASE_SHA="${1:-}"
HEAD_SHA="${2:-HEAD}"
EVENTS_FILE="contracts/scoregate-score/src/events.rs"

if [ -z "$BASE_SHA" ]; then
  echo "usage: $0 <base-sha> [head-sha]" >&2
  exit 2
fi

if ! git cat-file -e "$BASE_SHA^{commit}" 2>/dev/null; then
  echo "event schema check: base commit '$BASE_SHA' is unavailable" >&2
  exit 2
fi

schema_diff="$(git diff --unified=0 "$BASE_SHA" "$HEAD_SHA" -- "$EVENTS_FILE")"

# Event topic edits are schema edits. Require the explicit version marker in
# the same change so indexers cannot receive a new topic shape as version 1.
if printf '%s\n' "$schema_diff" \
  | grep -E '^[+-][^+-].*(publish\(|symbol_short!|Symbol::new)' \
  | grep -v '^[+-][^+-].*EVENT_VERSION' >/dev/null 2>&1; then
  if ! printf '%s\n' "$schema_diff" | grep -E '^[+-][^+-].*EVENT_VERSION' >/dev/null 2>&1; then
    echo "event schema check failed: event topic code changed without an EVENT_VERSION bump" >&2
    exit 1
  fi
fi

echo "event schema stability check passed"

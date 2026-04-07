#!/usr/bin/env bash
#
# Point git to the project's .githooks directory so commit-msg (and any
# future hooks) are picked up automatically.
#
# Usage: ./scripts/setup-hooks.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

git -C "$ROOT" config core.hooksPath .githooks
echo "Git hooks configured — using .githooks/ directory."

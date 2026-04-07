#!/usr/bin/env bash
#
# Generate a changelog from git commits since the last tag.
# Outputs markdown to stdout.
#
# Usage:
#   ./scripts/generate-changelog.sh          # since last tag
#   ./scripts/generate-changelog.sh v0.1.0   # since specific tag

set -euo pipefail

SINCE_TAG="${1:-}"

if [ -z "$SINCE_TAG" ]; then
  # Find the most recent tag
  SINCE_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
fi

if [ -z "$SINCE_TAG" ]; then
  # No tags exist — use all commits
  RANGE="HEAD"
  echo "## What's New"
  echo ""
  echo "Initial release."
  echo ""
else
  RANGE="${SINCE_TAG}..HEAD"
  echo "## What's Changed since ${SINCE_TAG}"
  echo ""
fi

# Categorize commits
declare -a features=()
declare -a fixes=()
declare -a other=()

while IFS= read -r line; do
  [ -z "$line" ] && continue

  # Extract short hash and message
  HASH="${line%% *}"
  MSG="${line#* }"

  case "$MSG" in
    feat:*|feat\(*|feature:*|add:*|add\ *)
      features+=("- ${MSG} (\`${HASH}\`)")
      ;;
    fix:*|fix\(*|bugfix:*)
      fixes+=("- ${MSG} (\`${HASH}\`)")
      ;;
    release:*|bump\ version*|Merge\ *)
      # Skip release/merge commits
      ;;
    *)
      other+=("- ${MSG} (\`${HASH}\`)")
      ;;
  esac
done < <(git log --oneline --no-decorate "$RANGE" 2>/dev/null)

if [ ${#features[@]} -gt 0 ]; then
  echo "### Features"
  printf '%s\n' "${features[@]}"
  echo ""
fi

if [ ${#fixes[@]} -gt 0 ]; then
  echo "### Fixes"
  printf '%s\n' "${fixes[@]}"
  echo ""
fi

# "Other" commits (docs, chore, refactor, etc.) are intentionally
# excluded from user-facing release notes.

# Stats
COMMIT_COUNT=$(git rev-list --count "$RANGE" 2>/dev/null || echo "0")
echo "---"
echo "*${COMMIT_COUNT} commits since ${SINCE_TAG:-initial}*"

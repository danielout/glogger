#!/bin/bash
# Strips a Player.log sample down to only LocalPlayer lines,
# excluding noisy ProcessMusicPerformance entries.
# Usage: ./scripts/minimize-player-log.sh <input.log>
# Output: <input>-minimized.log in the same directory

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <input.log>"
  exit 1
fi

input="$1"
dir=$(dirname "$input")
base=$(basename "$input" .log)
output="${dir}/${base}-minimized.log"

grep "LocalPlayer:" "$input" | grep -v "ProcessMusicPerformance" > "$output"

original=$(wc -l < "$input")
minimized=$(wc -l < "$output")
echo "Done: ${original} -> ${minimized} lines"
echo "Saved to: ${output}"

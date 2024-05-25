#! /usr/bin/env bash

# This script updates the README with a fresh test.sh benchmark result

DIR="$(dirname "$0")"

if [[ "${1:-__NOTHING__}" = "__NOTHING__" ]]; then
  "$DIR"/test.sh --help
  exit 1
fi

README_FILE="$DIR"/README.md
README="$(cat "$README_FILE")"

FIRST_PART_README="${README%%<!-- MAGICAL TEST START -->*}"
LAST_PART_README="${README##*<!-- MAGICAL TEST END -->}"

TEST_RESULT="$("$DIR"/test.sh "$1" 2>/dev/null)"
if [ $? -eq 0 ]; then
  echo -n "$FIRST_PART_README" > "$README_FILE"
  echo "<!-- MAGICAL TEST START -->" >> "$README_FILE"
  echo "\`\`\`" >> "$README_FILE"
  echo "$TEST_RESULT" | sed 's/\x1B\[[0-9;]\{1,\}[A-Za-z]//g' >> "$README_FILE"
  echo "\`\`\`" >> "$README_FILE"
  echo -n "<!-- MAGICAL TEST END -->" >> "$README_FILE"
  echo "$LAST_PART_README" >> "$README_FILE"
else
  echo "Error running test.sh. Please try again?"
  exit 1
fi

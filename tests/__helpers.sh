#! /usr/bin/env bash

# THIS FILE WILL BE RENAMED /tmp/yunohost-compat/__helpers.sh
# Start your tests with source /tmp/yunohost-compat/__helpers.sh

time="/usr/bin/time --format=%e -o /tmp/yunohost-compat/test.time"

red() {
  echo -ne "\033[0;31m$1\033[0m"
}

green() {
  echo -ne "\033[0;32m$1\033[0m"
}

startBench() {
  $time "$@" 2>&1
}

parsedOutput() {
  case "$1" in
    "json")
      echo "$2" | jq --sort-keys 'walk(if type == "array" then sort else . end)'
      ;;
    "yaml")
      echo "$2" | yq 'sort_keys(..)'
      ;;
    "plain")
      echo "$2"
      ;;
  esac
}

formatSuccess() {
  if [ $1 -eq 0 ]; then
    if [[ "${3:-NOTHING}" = "NOTHING" ]]; then
      echo -ne "$(green OK)"
    else
      echo -ne "$(green OK) (${3}s)"
    fi
  else
    tmp="$(mktemp)"
    # Still allow newline in error output
    echo "$OUTPUT" > "$tmp"
    echo -n "$(red $2) - $tmp"
    return 1
  fi
}

benchPythonRust() {
  diffFormat="$1"
  shift
  yunoCmd="$1"
  shift
  args="$@"

  startBench yunohost $yunoCmd "$@" &> /tmp/test.output
  pythonCode=$?
  pythonOutput="$(cat /tmp/test.output)"
  if OUTPUT="$pythonOutput" formatSuccess $pythonCode ERROR "$(cat /tmp/yunohost-compat/test.time)" > /tmp/test.status; then
    parsedOutput "$diffFormat" "$pythonOutput" &> /tmp/test.parsed.output
    pythonParsedCode=$?
    pythonParsedOutput="$(cat /tmp/test.parsed.output)"

    if [ ! $pythonParsedCode -eq 0 ]; then
      # Parsing failed, place output + parsing errors in single file
      echo -e "\n----------------- (COMMAND FINISHED HERE, PARSING ERRORS BELOW)" >> /tmp/test.output
      echo "$pythonParsedOutput" >> /tmp/test.output
      OUTPUT="$(cat /tmp/test.output)" formatSuccess 1 INVALID > /tmp/test.status
      pythonCode=1
    else
      echo "$pythonParsedOutput" > /tmp/yunohost-compat/python.parsed
      pythonCode=0
    fi
  else
    pythonCode=1
  fi
  pythonStatus="$(cat /tmp/test.status)"

  startBench /tmp/yunohost-compat/yunohost-$yunoCmd "$@" &> /tmp/test.output
  rustCode=$?
  rustOutput="$(cat /tmp/test.output)"
  if OUTPUT="$rustOutput" formatSuccess $rustCode ERROR "$(cat /tmp/yunohost-compat/test.time)" > /tmp/test.status; then
    parsedOutput "$diffFormat" "$rustOutput" > /tmp/test.parsed.output
    rustParsedCode=$?
    rustParsedOutput="$(cat /tmp/test.parsed.output)"

    if [ ! $rustParsedCode -eq 0 ]; then
      OUTPUT="$rustParsedOutput" formatSuccess 1 INVALID > /tmp/test.status
      rustCode=1
    else
      echo "$rustParsedOutput" > /tmp/yunohost-compat/rust.parsed
      rustCode=0
    fi
  else
    rustCode=1
  fi
  rustStatus="$(cat /tmp/test.status)"

  if [ $rustCode -eq 0 ] && [ $pythonCode -eq 0 ]; then
    # Compare output side by side, with python on the left
    output_diff="$(diff --side-by-side /tmp/yunohost-compat/python.parsed /tmp/yunohost-compat/rust.parsed)"
    OUTPUT="$output_diff" diffStatus="$(formatSuccess $? DIFF)"
  else
    diffStatus="SKIP"
  fi

  if [[ "${YUNOHOSTTESTOUTPUT:-__NOTHING__}" = "NOTHING" ]]; then
    # Not running inside __runner.sh, display output immediately
    column -s '|' -t << EOF
DIFF|PYTHON|RUST|COMMAND
$diffStatus|$pythonStatus|$rustStatus|yunohost $yunoCmd $args
EOF
  else
    # Running inside runner.sh... save output for later
    echo "$diffStatus|$pythonStatus|$rustStatus|yunohost $yunoCmd $args" >> "$YUNOHOSTTESTOUTPUT"
  fi
}

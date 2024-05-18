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
      echo "$2" | jq --sort-keys '.' > /tmp/parsed.output
      ;;
    "yaml")
      echo "$2" | yq 'sort_keys(..)' > /tmp/parsed.output
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
    echo -n "$OUTPUT" > "$tmp"
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

  pythonOutput="$(startBench yunohost $yunoCmd "$@")"
  if OUTPUT="$pythonOutput" pythonStatus="$(formatSuccess $? ERROR "$(cat /tmp/yunohost-compat/test.time)")"; then
    if ! pythonParsedOutput="(parsedOutput "$diffFormat" "$pythonOutput")"; then
      OUTPUT="$pythonParsedOutput" pythonStatus="$(formatSuccess 1 INVALID)"
      pythonCode=1
    else
      echo "$pythonParsedOutput" > /tmp/yunohost-compat/python.parsed
      pythonCode=0
    fi
  else
    pythonCode=1
  fi

  rustOutput="$(startBench /tmp/yunohost-compat/yunohost-$yunoCmd "$@")"
  if OUTPUT="$rustOutput" rustStatus="$(formatSuccess $? ERROR "$(cat /tmp/yunohost-compat/test.time)")"; then
    if ! rustParsedOutput="(parsedOutput "$diffFormat" "$pythonOutput")"; then
      OUTPUT="$rustParsedOutput" rustStatus="$(formatSuccess 1 INVALID)"
      rustCode=1
    else
      echo "$rustParsedOutput" > /tmp/yunohost-compat/rust.parsed
      rustCode=0
    fi
  else
    rustCode=1
  fi

  if [ $rustCode -eq 0 ] && [ $pythonCode -eq 0 ]; then
    output_diff="$(diff /tmp/yunohost-compat/python.parsed /tmp/yunohost-compat/rust.parsed)"
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

#! /usr/bin/env bash

export YUNOHOSTTESTOUTPUT="$(mktemp)"

echo __runner.sh "$@"

for test in "$@"; do
  /tmp/yunohost-compat/compat/$test
done

column -s '|' -t << EOF
DIFF|PYTHON|RUST|COMMAND
$(cat "$YUNOHOSTTESTOUTPUT")
EOF

rm "$YUNOHOSTTESTOUTPUT"

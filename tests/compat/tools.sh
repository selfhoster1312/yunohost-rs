#! /usr/bin/env bash

source /tmp/yunohost-compat/__helpers.sh

benchPythonRust json "tools" regen-conf --list-pending --json
benchPythonRust json "tools" regen-conf --list-pending --with-diff --json

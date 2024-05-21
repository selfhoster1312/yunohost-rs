#! /usr/bin/env bash

source /tmp/yunohost-compat/__helpers.sh

# Panel list options
benchPythonRust json "settings" get --json security

# Section list options
benchPythonRust json "settings" get --json security.webadmin

# Single option
benchPythonRust json "settings" get --json security.webadmin.webadmin_allowlist_enabled

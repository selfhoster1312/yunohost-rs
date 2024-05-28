#! /usr/bin/env bash

source /tmp/yunohost-compat/__helpers.sh

# Panel list options
benchPythonRust json "settings" get --json security
benchPythonRust json "settings" get --full --json security

# Section list options
benchPythonRust json "settings" get --json security.webadmin
benchPythonRust json "settings" get --full --json security.webadmin

# Single option
benchPythonRust json "settings" get --json security.webadmin.webadmin_allowlist_enabled
benchPythonRust json "settings" get --full --json security.webadmin.webadmin_allowlist_enabled

# All options
benchPythonRust json "settings" list --json
benchPythonRust json "settings" list --full --json

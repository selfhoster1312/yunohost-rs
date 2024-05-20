#! /usr/bin/env bash

source /tmp/yunohost-compat/__helpers.sh

# Panel list options
benchPythonRust plain "settings" get security

# Section list options
benchPythonRust plain "settings" get security.webadmin

# Single option
benchPythonRust plain "settings" get security.webadmin.webadmin_allowlist_enabled

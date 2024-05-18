#! /usr/bin/env bash

source /tmp/yunohost-compat/__helpers.sh

benchPythonRust json "hook" list --json conf_regen

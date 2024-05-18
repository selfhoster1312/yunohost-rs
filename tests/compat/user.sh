#! /usr/bin/env bash

source /tmp/yunohost-compat/__helpers.sh

benchPythonRust json "user" list --json
benchPythonRust json "user" info --json test2

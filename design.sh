#!/bin/bash

# Dev runner with the Script Kit Dev Style Tool sidecar enabled.
# Passes through to dev.sh so the normal cargo-watch workflow stays unchanged.

set -e

cd "$(dirname "$0")"

export SCRIPT_KIT_STYLE_DEVTOOLS=1
export SCRIPT_KIT_DEV_FORCE_RELAUNCH=1

echo "[design.sh] enabling SCRIPT_KIT_STYLE_DEVTOOLS=1"
echo "[design.sh] enabling SCRIPT_KIT_DEV_FORCE_RELAUNCH=1"
exec ./dev.sh "$@"

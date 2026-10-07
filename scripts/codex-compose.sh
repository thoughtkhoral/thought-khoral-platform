#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu
platform_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
[ "$#" -gt 0 ] || { printf 'Usage: %s <compose arguments, e.g. config>\n' "$0" >&2; exit 2; }
python3 "$platform_dir/scripts/check-codex-inputs.py"
export THOUGHT_KHORAL_PLATFORM_ROOT=$platform_dir
exec podman-compose -f "$platform_dir/compose.yaml" -f "$platform_dir/compose.codex.yaml" "$@"

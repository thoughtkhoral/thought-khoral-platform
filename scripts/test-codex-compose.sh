#!/bin/sh
# Compose parsing and startup-validation tests; no services are activated.
set -eu
cd "$(dirname "$0")/.."
python3 scripts/tests/codex-compose.py
python3 scripts/tests/codex-inputs.py

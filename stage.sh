#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$root/scripts/stage-built.sh" "${1:-dist}" "${2:-}"

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

exec cargo build --manifest-path "$script_dir/Cargo.toml" "$@"

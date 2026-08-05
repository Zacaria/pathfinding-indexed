#!/bin/sh

set -eu

msrv=$(awk -F '"' '/^rust-version =/ {print $2}' Cargo.toml)

if [ -z "$msrv" ]; then
  echo "rust-version not found in Cargo.toml" >&2
  exit 1
fi

if ! grep -F "Rust $msrv" src/lib.rs >/dev/null 2>&1; then
  echo "MSRV $msrv not found in src/lib.rs" >&2
  exit 1
fi

printf 'MSRV %s is consistent between Cargo.toml and src/lib.rs\n' "$msrv"

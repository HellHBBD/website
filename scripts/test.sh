#!/usr/bin/env bash
set -euo pipefail

echo "Running tests..."
cargo test

echo "Running clippy..."
cargo clippy

if [ -f deploy/Caddyfile ]; then
  echo "Validating Caddyfile..."
  caddy validate --config deploy/Caddyfile
fi

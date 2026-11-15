#!/bin/sh
set -e
cd "$(dirname "$0")"
cargo run -q -- build
set -a; . ~/.pi/agent/secrets/agentrambench-cloudflare.env; set +a
npx --yes wrangler pages deploy dist --project-name agentrambench --branch main --commit-hash "$(git rev-parse HEAD)" --commit-dirty=true

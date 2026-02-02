#!/bin/bash
ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT_DIR"

# Prefer processor-specific env (keypair path, worker counts, Solana RPC, etc.)
ENV_FILE="services/processor/.env"
if [ -f "$ENV_FILE" ]; then
	export $(grep -v '^#' "$ENV_FILE" | xargs)
else
	export $(grep -v '^#' .env | xargs)
fi

# Run from the processor service directory so any relative paths in env are valid.
cd "$ROOT_DIR/services/processor"
exec "$ROOT_DIR/target/release/processor"

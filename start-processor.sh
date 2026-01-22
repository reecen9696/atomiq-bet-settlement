#!/bin/bash
cd "$(dirname "$0")"
export $(grep -v '^#' .env | xargs)
exec ./target/release/processor

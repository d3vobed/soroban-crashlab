#!/bin/bash
set -e

# Build the soroban-example contract for wasm32 target
# This script builds the contract to a .wasm file that can be deployed to Soroban

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACT_DIR="$(cd "$SCRIPT_DIR/../../contracts/soroban-example" && pwd)"

echo "Building soroban-example contract for wasm32-unknown-unknown..."
cd "$CONTRACT_DIR"

cargo build --target wasm32-unknown-unknown --release

echo "Build complete!"
echo "WASM file location: $CONTRACT_DIR/target/wasm32-unknown-unknown/release/soroban_example.wasm"

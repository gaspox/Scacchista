#!/bin/bash
# UCI wrapper for Scacchista chess engine
# For use with Lucas Chess and other UCI-compatible GUIs

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Path to the release binary
ENGINE_BINARY="$SCRIPT_DIR/target/release/scacchista"

# Check if the engine binary exists
if [ ! -f "$ENGINE_BINARY" ]; then
    echo "Error: Engine binary not found at $ENGINE_BINARY" >&2
    echo "Please build with: cargo build --release" >&2
    exit 1
fi

# Check if the binary is executable
if [ ! -x "$ENGINE_BINARY" ]; then
    echo "Error: Engine binary is not executable" >&2
    chmod +x "$ENGINE_BINARY" 2>/dev/null
    if [ $? -ne 0 ]; then
        echo "Failed to make binary executable. Please run: chmod +x $ENGINE_BINARY" >&2
        exit 1
    fi
fi

# Execute the engine
# stdin/stdout are automatically connected for UCI communication
exec "$ENGINE_BINARY"

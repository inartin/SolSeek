#!/bin/bash

# Check if the binary exists
if [ ! -f "./target/release/solana_vanity_generator" ]; then
    echo "Binary not found. Building in release mode..."
    cargo build --release
fi

# Run the generator with all passed arguments
./target/release/solana_vanity_generator "$@"
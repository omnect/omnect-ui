#!/bin/bash
# Generate TypeScript types from Rust
#
# We delete the generated CommonJS .js files so that Vite/Rollup uses 
# the TypeScript source directly (which has proper 'export' statements).
# This aligns with the Dockerfile build process.

set -e

echo "Generating TypeScript types from Rust..."
cargo build -p shared_types

echo "Cleaning up generated CommonJS files..."
# Remove .js files to force Vite to use .ts sources
find src/shared_types/generated/typescript -name "*.js" -delete

echo "TypeScript types generated successfully!"
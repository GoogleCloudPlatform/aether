#!/bin/bash
# Build the runtime library

echo "Building AetherScript runtime library..."

# Create a temporary directory for object files
mkdir -p target/runtime

# Compile the runtime module to object file
rustc --crate-type staticlib \
      --crate-name aether_runtime \
      -C opt-level=3 \
      -C codegen-units=1 \
      -o target/runtime/libaether_runtime.a \
      src/runtime/mod.rs

echo "Runtime library built at target/runtime/libaether_runtime.a"
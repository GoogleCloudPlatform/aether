#!/bin/bash
# Script to compile and run AetherScript examples

if [ $# -eq 0 ]; then
    echo "Usage: $0 <example_name>"
    echo "Example: $0 simple"
    echo ""
    echo "Available examples:"
    ls examples/*.aether | xargs -n1 basename | sed 's/\.aether$//' | sort
    exit 1
fi

EXAMPLE_NAME=$1
EXAMPLE_FILE="examples/${EXAMPLE_NAME}.aether"

if [ ! -f "$EXAMPLE_FILE" ]; then
    echo "Error: Example file '$EXAMPLE_FILE' not found"
    exit 1
fi

echo "Compiling $EXAMPLE_FILE..."

# Build the compiler if needed
cargo build --release --quiet

# Compile the example
./target/release/aether compile "$EXAMPLE_FILE" -o "$EXAMPLE_NAME"

if [ $? -eq 0 ]; then
    echo "Running $EXAMPLE_NAME..."
    ./"$EXAMPLE_NAME"
    
    # Clean up
    rm -f "$EXAMPLE_NAME"
else
    echo "Compilation failed"
    exit 1
fi
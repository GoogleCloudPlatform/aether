#!/bin/bash
# Script to compile AetherScript programs with proper linking

if [ $# -ne 2 ]; then
    echo "Usage: $0 <output_name> <input_file>"
    exit 1
fi

OUTPUT=$1
INPUT=$2

# Compile the AetherScript program (compile only, no linking)
cargo run -- compile -c -o "$OUTPUT" "$INPUT"

if [ $? -eq 0 ]; then
    # Link with the runtime library
    echo "Linking with runtime library..."
    clang "$OUTPUT.o" -o "$OUTPUT" \
        -L./runtime/target/debug \
        -laether_runtime \
        -lSystem \
        -framework CoreFoundation
        
    if [ $? -eq 0 ]; then
        echo "Successfully created executable: $OUTPUT"
    else
        echo "Linking failed"
        exit 1
    fi
else
    echo "Compilation failed"
    exit 1
fi
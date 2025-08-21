#!/bin/bash
# Copyright 2025 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
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

# Run an AetherScript standard library example

# Check arguments
if [ $# -eq 0 ]; then
    echo "Usage: $0 <example_name>"
    echo ""
    echo "Available examples:"
    echo "  stdlib_demo              - Basic standard library features"
    echo "  stdlib_file_processing   - File I/O and text processing"
    echo "  stdlib_data_analysis     - Statistical analysis"
    echo "  stdlib_concurrent_tasks  - Concurrent programming"
    echo "  stdlib_time_operations   - Time and date manipulation"
    echo "  stdlib_network_client    - Network operations"
    echo "  stdlib_resource_management - Resource handling"
    exit 1
fi

EXAMPLE_NAME=$1
COMPILER="./target/release/aether"
EXAMPLE_PATH="examples/${EXAMPLE_NAME}.aether"

# Check if compiler exists
if [ ! -f "$COMPILER" ]; then
    echo "Error: Compiler not found at $COMPILER"
    echo "Please build the compiler first with: cargo build --release"
    exit 1
fi

# Check if example exists
if [ ! -f "$EXAMPLE_PATH" ]; then
    echo "Error: Example '${EXAMPLE_NAME}' not found at $EXAMPLE_PATH"
    echo "Run without arguments to see available examples."
    exit 1
fi

# Create temp directory for output
TEMP_DIR="/tmp/aether_examples"
mkdir -p "$TEMP_DIR"
OUTPUT_PATH="$TEMP_DIR/${EXAMPLE_NAME}"

echo "Compiling ${EXAMPLE_NAME}..."
if $COMPILER compile "$EXAMPLE_PATH" -o "$OUTPUT_PATH"; then
    echo "Running ${EXAMPLE_NAME}..."
    echo "===================="
    "$OUTPUT_PATH"
    EXIT_CODE=$?
    echo "===================="
    echo "Example finished with exit code: $EXIT_CODE"
    
    # Cleanup
    rm -f "$OUTPUT_PATH"
else
    echo "Compilation failed!"
    exit 1
fi
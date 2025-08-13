#!/bin/bash

# Test script for AetherScript Text Analyzer Example
set -e

echo "Testing AetherScript Text Analyzer Example..."

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Run the build script
echo "Running build..."
./build.sh

# Check if the build was successful
if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Text Analyzer example built and tested successfully!"
    echo ""
    echo "Files created:"
    echo "- main.aether (AetherScript source)"
    echo "- build.sh (build script)"
    echo "- README.md (documentation)"
    echo "- output/text_analyzer (compiled executable)"
    echo ""
    echo "To run manually:"
    echo "  cd examples/text_analyzer"
    echo "  ./build.sh"
    echo ""
else
    echo "❌ Build/test failed"
    exit 1
fi
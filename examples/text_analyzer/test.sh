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
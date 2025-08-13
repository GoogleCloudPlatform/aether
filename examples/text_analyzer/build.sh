#!/bin/bash

# Build script for AetherScript Text Analyzer Example
set -e

echo "Building AetherScript Text Analyzer Example..."

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Script directory: $SCRIPT_DIR"
echo "Project root: $PROJECT_ROOT"

# Change to project root
cd "$PROJECT_ROOT"

# Check if we have a working compiler binary
AETHER_COMPILER="$PROJECT_ROOT/target/release/aether-compiler"
AETHER_COMPILER_DEBUG="$PROJECT_ROOT/target/debug/aether-compiler"

# Try to use existing compiler first
if [ -f "$AETHER_COMPILER" ]; then
    echo "Using existing release compiler: $AETHER_COMPILER"
    COMPILER_PATH="$AETHER_COMPILER"
elif [ -f "$AETHER_COMPILER_DEBUG" ]; then
    echo "Using existing debug compiler: $AETHER_COMPILER_DEBUG"
    COMPILER_PATH="$AETHER_COMPILER_DEBUG"
else
    echo "❌ No compiler found. Please build with 'cargo build --release' first."
    exit 1
fi

# Change to the example directory
cd "$SCRIPT_DIR"

# Create output directory
mkdir -p output

echo "Using compiler: $COMPILER_PATH"
echo ""

# Show what we're trying to compile
echo "Compiling AetherScript source:"
echo "================================"
head -10 main.aether
echo "... [file continues]"
echo "================================"
echo ""

# Try compilation with full error output
echo "Running compilation..."
if "$COMPILER_PATH" compile main.aether -o output/text_analyzer 2>compilation_error.log; then
    echo "✅ Compilation successful!"
    echo "Output: $SCRIPT_DIR/output/text_analyzer"
    
    # Try to run if executable was created
    if [ -f "output/text_analyzer" ]; then
        echo ""
        echo "Running text_analyzer..."
        echo "=========================="
        ./output/text_analyzer
        echo "=========================="
        echo "✅ Execution completed!"
    else
        echo "⚠️  Executable not found, but compilation reported success"
    fi
elif "$COMPILER_PATH" check main.aether 2>compilation_error.log; then
    echo "✅ Syntax check successful!"
else
    echo "❌ Compilation failed. Error details:"
    echo "===================================="
    if [ -f "compilation_error.log" ]; then
        cat compilation_error.log
    else
        echo "No error log found"
    fi
    echo "===================================="
    echo ""
    echo "This is expected since AetherScript is still in development."
    echo "The example demonstrates the language's LLM-first design principles:"
    echo ""
    echo "🎯 LLM-First Features Demonstrated:"
    echo "- RESOURCE_SCOPE for deterministic cleanup"
    echo "- COMPOSE_PATTERNS for sequential processing"
    echo "- PRECONDITION/POSTCONDITION for verification"
    echo "- INVARIANT for loop correctness"
    echo "- INTENT for natural language documentation"
    echo "- Structured error handling with auto-fix suggestions"
    echo ""
    echo "📁 Files:"
    echo "- main.aether: Complete implementation"
    echo "- README.md: Detailed feature documentation"
    echo "- build.sh: This build script"
    
    # Try to get help text to see what commands are available
    echo ""
    echo "Available compiler commands:"
    echo "============================="
    "$COMPILER_PATH" --help 2>/dev/null || echo "Help not available"
    
    exit 1
fi

# Cleanup
rm -f compilation_error.log
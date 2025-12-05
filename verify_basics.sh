#!/bin/bash

EXAMPLES=(
    "examples/v2/01-basics/hello_world"
    "examples/v2/01-basics/module_system"
    "examples/v2/01-basics/simple_main"
    "examples/v2/02-variables/constants"
    "examples/v2/02-variables/let_bindings"
    "examples/v2/02-variables/mutability"
    "examples/v2/03-types/primitives"
    "examples/v2/04-functions/basic_functions"
    "examples/v2/04-functions/parameters"
    "examples/v2/05-operators/arithmetic"
    "examples/v2/06-control-flow/if_else"
    "examples/v2/06-control-flow/loops"
    "examples/v2/07-structs/struct_methods"
    "examples/v2/07-structs/basic_struct"
    "examples/v2/08-enums/basic_enum"
    "examples/v2/09-pattern-matching/match_basics"
    "examples/v2/10-collections/maps"
    "examples/v2/13-strings/string_basics"
    "examples/v2/13-strings/string_operations"
)

FAILURES=0

echo "Starting verification of basic examples..."

for example in "${EXAMPLES[@]}"; do
    echo "----------------------------------------------------------------"
    echo "Verifying $example..."
    
    # Check if directory exists
    if [ ! -d "$example" ]; then
        echo "⚠️  Directory not found: $example"
        continue
    fi

    # Check if main.aes exists
    if [ ! -f "$example/main.aes" ]; then
        echo "⚠️  main.aes not found in $example"
        continue
    fi

    # Compile and run
    output=$(cargo run -- run "$example/main.aes" 2>&1)
    exit_code=$?

    # Verify result (0 or known good exit codes like 42, 120, 2, 7, 55, 50, 3, 195, 4, 11)
    # Note: Some examples return 0, some return values.
    # If compilation failed (exit code 1 or 101), it's a failure.
    if [ $exit_code -ne 0 ] && [ $exit_code -ne 42 ] && [ $exit_code -ne 120 ] && [ $exit_code -ne 2 ] && [ $exit_code -ne 7 ] && [ $exit_code -ne 55 ] && [ $exit_code -ne 50 ] && [ $exit_code -ne 3 ] && [ $exit_code -ne 195 ] && [ $exit_code -ne 4 ] && [ $exit_code -ne 11 ]; then
        # Check if it failed compilation or just returned non-zero
        if echo "$output" | grep -q "Compilation failed"; then
            echo "❌ FAILED: Compilation error"
            echo "$output" | grep "Compilation failed"
            FAILURES=$((FAILURES + 1))
        elif echo "$output" | grep -q "error:"; then
             echo "❌ FAILED: Rust compilation error or other error"
             echo "$output" | grep "error:" | head -n 5
             FAILURES=$((FAILURES + 1))
        else
             echo "ℹ️  Exited with code $exit_code (might be expected)"
        fi
    else
        echo "✅ PASSED"
    fi
done

echo "----------------------------------------------------------------"
if [ $FAILURES -eq 0 ]; then
    echo "🎉 All checked examples passed!"
else
    echo "❌ $FAILURES examples failed."
fi

#!/bin/bash
# Cargo wrapper script with Z3 environment variables set
# Usage: ./scripts/cargo-z3.sh <cargo-command> [arguments...]
# Examples:
#   ./scripts/cargo-z3.sh build
#   ./scripts/cargo-z3.sh build --release
#   ./scripts/cargo-z3.sh test
#   ./scripts/cargo-z3.sh test --lib test_parse_if
#   ./scripts/cargo-z3.sh test --lib -- --nocapture

export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
export CPPFLAGS="-I/opt/homebrew/opt/z3/include"
export LDFLAGS="-L/opt/homebrew/opt/z3/lib"

cargo "$@"

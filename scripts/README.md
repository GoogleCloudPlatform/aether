# AetherScript Build and Development Scripts

This directory contains utility scripts for building, testing, and developing AetherScript.

## Build Scripts

### `build_runtime.sh`
Builds the AetherScript runtime library using rustc.
- Creates static library at `target/runtime/libaether_runtime.a`
- Optimized build with `-C opt-level=3`

### `compile_aether.sh`
Legacy compilation script for AetherScript programs.
- Compiles individual .aether files
- Links with runtime library

## Development Scripts

### `fix_stdlib_modules.sh`
Fixes stdlib module external function declarations.
- Updates external functions to use `create_external_function_named`
- Processes all stdlib modules (io, collections, math, etc.)

### `fix_symbols.sh`
Symbol resolution fix script.
- Resolves symbol linking issues
- Updates external function bindings

## Testing Scripts

### `test_review.sh`
Comprehensive test review and analysis script.
- Reviews test compilation status
- Provides detailed error analysis
- Generates test status reports

### `run_example.sh`
Runs individual AetherScript example programs.
- Compiles and executes example files
- Handles command-line arguments

### `run_stdlib_example.sh`
Runs standard library example programs.
- Tests stdlib functionality
- Validates library integration

## Demo Scripts

### `run_blog_demo.sh`
HTTP blog server demonstration script.
- Shows available blog endpoints
- Provides instructions for running actual server
- Demonstrates AetherScript web capabilities

## Usage

Make sure scripts are executable:
```bash
chmod +x scripts/*.sh
```

Run from project root:
```bash
./scripts/build_runtime.sh
./scripts/run_example.sh examples/hello_world.aether
./scripts/run_blog_demo.sh
```

## Dependencies

Scripts require:
- Rust toolchain (rustc, cargo)
- AetherScript compiler built with `cargo build --release`
- Bash shell environment
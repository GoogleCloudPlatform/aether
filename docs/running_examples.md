# Running AetherScript Examples

## Building the Compiler

First, build the AetherScript compiler:

```bash
cargo build --release
```

## Running Examples

### Method 1: Using the run_example.sh script

```bash
./run_example.sh simple
./run_example.sh arithmetic_operations
./run_example.sh pattern_match_simple
```

### Method 2: Direct compilation

```bash
# Compile an example
./target/release/aether compile examples/simple.aether -o simple

# Run the compiled program
./simple

# Clean up
rm simple
```

### Method 3: Compile with options

```bash
# Compile with debug info
./target/release/aether compile examples/simple.aether -o simple --debug

# Compile with specific optimization level
./target/release/aether compile examples/simple.aether -o simple -O3

# Compile to object file only (no linking)
./target/release/aether compile examples/simple.aether -c -o simple.o
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test test_function_metadata_parsing
cargo test test_mutability_checking
```

## Interactive Development

For development and testing, you can also:

1. Use the REPL (if implemented):
   ```bash
   ./target/release/aether repl
   ```

2. Check syntax without full compilation:
   ```bash
   ./target/release/aether check examples/simple.aether
   ```

3. View intermediate representations:
   ```bash
   ./target/release/aether compile examples/simple.aether --keep-intermediates
   ```
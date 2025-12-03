# Aether Project Structure Convention

Aether projects follow a Go-inspired package-per-directory structure with tests alongside code.

## Standard Layout

```
project_name/
├── aether.toml              # Project manifest
├── README.md                # Project documentation
├── cmd/                     # Binary entry points
│   └── project_name/
│       └── main.aether      # Main entry point
├── package_a/               # Package directory = package name
│   ├── package_a.aether     # Primary implementation
│   ├── helper.aether        # Additional implementation files
│   └── package_a_test.aether    # Tests for this package
├── package_b/
│   ├── package_b.aether
│   ├── subcomponent.aether
│   └── package_b_test.aether
└── internal/                # Private packages (not importable externally)
    └── utils/
        ├── utils.aether
        └── utils_test.aether
```

## Conventions

### Directory = Package
- Each directory is a package
- Directory name is the package name
- Import path: `project_name/package_name`

### File Naming
| Type | Pattern | Example |
|------|---------|---------|
| Implementation | `name.aether` | `tokenizer.aether` |
| Tests | `name_test.aether` | `tokenizer_test.aether` |
| Entry point | `main.aether` | `cmd/starling/main.aether` |

### Package Structure
```
tokenizer/
├── tokenizer.aether     # Primary file (same name as directory)
├── bpe.aether          # Additional implementation
├── vocab.aether        # Additional implementation
└── tokenizer_test.aether   # All tests for this package
```

### Test Files
- Tests live alongside the code they test
- Named `{package}_test.aether` or `{file}_test.aether`
- Test functions: `func test_feature_name() -> Bool`

### Binary Entry Points
- Live in `cmd/{binary_name}/main.aether`
- One directory per binary
- Multiple binaries supported:
  ```
  cmd/
  ├── server/
  │   └── main.aether
  └── cli/
      └── main.aether
  ```

### Internal Packages
- `internal/` directory for private packages
- Cannot be imported by external projects
- Use for implementation details

## Project Manifest (aether.toml)

```toml
[project]
name = "project_name"
version = "0.1.0"
description = "Project description"

[build]
entry = "cmd/project_name/main.aether"

[dependencies]
# package_name = "version"

[dev-dependencies]
# test_framework = "version"
```

## Import Syntax

```aether
// Import entire package
import starling/tokenizer;

// Import specific items
import starling/tokenizer { encode, decode };

// Aliased import
import starling/tokenizer as tok;
```

## Example: Starling Project

```
starling/
├── aether.toml
├── README.md
├── ARCHITECTURE.md
├── TASKS.md
├── cmd/
│   └── starling/
│       └── main.aether          # HTTP server entry point
├── tokenizer/
│   ├── tokenizer.aether         # BPE tokenizer
│   ├── vocab.aether             # Vocabulary loading
│   └── tokenizer_test.aether    # Tokenizer tests
├── sampler/
│   ├── sampler.aether           # Sampling pipeline
│   ├── temperature.aether       # Temperature scaling
│   └── sampler_test.aether
├── kvcache/
│   ├── kvcache.aether           # KV cache manager
│   ├── arena.aether             # Arena allocator
│   └── kvcache_test.aether
├── scheduler/
│   ├── scheduler.aether         # Request scheduler
│   ├── batch.aether             # Micro-batching
│   └── scheduler_test.aether
├── model/
│   ├── model.aether             # Model runtime
│   ├── attention.aether         # Attention mechanism
│   ├── ffn.aether               # Feed-forward network
│   └── model_test.aether
├── http/
│   ├── http.aether              # HTTP server
│   ├── routes.aether            # API routes
│   └── http_test.aether
└── telemetry/
    ├── telemetry.aether         # Metrics and logging
    └── telemetry_test.aether
```

## Running Tests

```bash
# Run all tests
aether test

# Run specific package tests
aether test tokenizer/

# Run with verbose output
aether test -v
```

## Building

```bash
# Build default binary
aether build

# Build specific binary
aether build cmd/starling

# Build with optimizations
aether build --release
```

# Aether Separate Compilation Architecture

## Overview

This document specifies the separate compilation system for Aether, enabling:
- Pre-compiled standard library modules
- Faster incremental builds
- Clean module boundaries
- Future: package distribution

## Compilation Model

```
                    COMPILE PHASE                          LINK PHASE

┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ module.aether│────▶│  Compiler   │────▶│ module.o    │──┐
└─────────────┘     │             │     └─────────────┘  │
                    │             │     ┌─────────────┐  │     ┌─────────────┐
                    │             │────▶│ module.abi  │  ├────▶│   Linker    │
                    └─────────────┘     └─────────────┘  │     │             │
                                              │          │     │      ▼      │
                    ┌─────────────┐           │          │     │  Executable │
                    │ dep.abi     │───────────┘          │     └─────────────┘
                    │ (read-only) │                      │
                    └─────────────┘                      │
                                                         │
                    ┌─────────────┐                      │
                    │ stdlib/*.o  │──────────────────────┘
                    └─────────────┘
```

### File Types

| Extension | Purpose | Contents |
|-----------|---------|----------|
| `.aether` | Source code | Aether source |
| `.abi` | Module interface | Type signatures, contracts, metadata |
| `.o` | Object code | Compiled machine code |
| `.mir` | Optional: Generic bodies | MIR for monomorphization |

## ABI File Format Specification

### Version and Header

```json
{
  "abi_version": "1.0.0",
  "aether_version": "0.1.0",
  "module": {
    "name": "std.stringview",
    "path": "stdlib/stringview.aether",
    "checksum": "sha256:abc123..."
  },
  "dependencies": [...],
  "exports": {...},
  "types": {...},
  "traits": {...},
  "impls": {...}
}
```

### Dependencies Section

```json
"dependencies": [
  {
    "module": "std.io",
    "version_constraint": ">=1.0.0",
    "imports": [
      {"name": "print", "as": null},
      {"name": "read_line", "as": "readline"}
    ]
  }
]
```

### Exports Section

#### Functions

```json
"exports": {
  "functions": [
    {
      "name": "view_from_string",
      "signature": {
        "generic_params": [],
        "where_clauses": [],
        "parameters": [
          {"name": "s", "type": "String", "mode": "owned"}
        ],
        "return_type": "Int64",
        "is_variadic": false
      },
      "kind": {
        "type": "extern",
        "library": "aether_runtime",
        "symbol": "stringview_from_string",
        "calling_convention": "C"
      },
      "contracts": {
        "preconditions": [],
        "postconditions": [
          {"expr": "result != 0", "message": "Returns valid handle"}
        ],
        "verified": true
      },
      "attributes": ["export", "inline_never"],
      "source_location": {"line": 28, "column": 1}
    },
    {
      "name": "equals",
      "signature": {
        "generic_params": [],
        "where_clauses": [],
        "parameters": [
          {"name": "a", "type": "String", "mode": "borrowed"},
          {"name": "b", "type": "String", "mode": "borrowed"}
        ],
        "return_type": "Int",
        "is_variadic": false
      },
      "kind": {
        "type": "native",
        "symbol": "std.stringview.equals",
        "has_mir": true
      },
      "contracts": {
        "preconditions": [],
        "postconditions": [
          {"expr": "result == 0 || result == 1", "message": "Returns boolean"}
        ],
        "verified": true
      },
      "attributes": ["export", "pure"],
      "source_location": {"line": 150, "column": 1}
    }
  ]
}
```

#### Generic Functions (require MIR storage)

```json
{
  "name": "map",
  "signature": {
    "generic_params": [
      {"name": "T", "kind": "type"},
      {"name": "U", "kind": "type"}
    ],
    "where_clauses": [],
    "parameters": [
      {"name": "arr", "type": "Array<T>", "mode": "borrowed"},
      {"name": "f", "type": "Fn(T) -> U", "mode": "owned"}
    ],
    "return_type": "Array<U>",
    "is_variadic": false
  },
  "kind": {
    "type": "generic",
    "mir_offset": 0,
    "mir_length": 1024
  },
  "contracts": {...},
  "attributes": ["export"],
  "source_location": {"line": 200, "column": 1}
}
```

### Types Section

#### Structs

```json
"types": {
  "structs": [
    {
      "name": "StringView",
      "generic_params": [],
      "where_clauses": [],
      "fields": [
        {"name": "ptr", "type": "Int64", "visibility": "private"},
        {"name": "len", "type": "Int", "visibility": "private"},
        {"name": "parent", "type": "Int64", "visibility": "private"}
      ],
      "attributes": ["derive(Copy)"],
      "invariants": [
        {"expr": "len >= 0", "message": "Length is non-negative"}
      ],
      "source_location": {"line": 50, "column": 1}
    }
  ],
  "enums": [
    {
      "name": "Option",
      "generic_params": [{"name": "T", "kind": "type"}],
      "variants": [
        {"name": "Some", "fields": [{"name": "value", "type": "T"}]},
        {"name": "None", "fields": []}
      ],
      "source_location": {"line": 100, "column": 1}
    }
  ],
  "type_aliases": [
    {
      "name": "ViewHandle",
      "target": "Int64",
      "source_location": {"line": 45, "column": 1}
    }
  ]
}
```

### Traits Section

```json
"traits": [
  {
    "name": "Eq",
    "generic_params": [],
    "super_traits": [],
    "methods": [
      {
        "name": "eq",
        "signature": {
          "parameters": [
            {"name": "self", "type": "Self", "mode": "borrowed"},
            {"name": "other", "type": "Self", "mode": "borrowed"}
          ],
          "return_type": "Bool"
        },
        "default_impl": false
      }
    ],
    "axioms": [
      {
        "name": "reflexivity",
        "quantifiers": [{"var": "x", "type": "Self", "kind": "forall"}],
        "expr": "x.eq(x) == true",
        "source_location": {"line": 120, "column": 5}
      },
      {
        "name": "symmetry",
        "quantifiers": [
          {"var": "x", "type": "Self", "kind": "forall"},
          {"var": "y", "type": "Self", "kind": "forall"}
        ],
        "expr": "x.eq(y) == y.eq(x)",
        "source_location": {"line": 125, "column": 5}
      }
    ],
    "source_location": {"line": 110, "column": 1}
  }
]
```

### Impl Section

```json
"impls": [
  {
    "trait": "Eq",
    "for_type": "String",
    "generic_params": [],
    "where_clauses": [],
    "methods": [
      {
        "name": "eq",
        "symbol": "std.string.String.eq"
      }
    ],
    "source_location": {"line": 200, "column": 1}
  }
]
```

## MIR Storage Format

For generic functions that require monomorphization, we store the MIR (Mid-level IR) in a separate section or file.

### MIR File Format (.mir)

```
AETHER_MIR_V1
module: std.collections
functions: 3
---
[function: map<T, U>]
blocks: 5
locals: 8
---
bb0:
  _1 = param[0]  // arr: Array<T>
  _2 = param[1]  // f: Fn(T) -> U
  _3 = call array_len(_1)
  _4 = call array_new<U>(_3)
  _5 = const 0
  goto bb1
bb1:
  _6 = lt(_5, _3)
  switch _6 [true: bb2, false: bb4]
bb2:
  _7 = call array_get<T>(_1, _5)
  _8 = call _2(_7)
  call array_set<U>(_4, _5, _8)
  goto bb3
bb3:
  _5 = add(_5, 1)
  goto bb1
bb4:
  return _4
---
[function: filter<T>]
...
```

### Alternative: Embed MIR in ABI JSON

```json
{
  "name": "map",
  "kind": {
    "type": "generic",
    "mir": {
      "blocks": [
        {
          "id": "bb0",
          "statements": [
            {"kind": "assign", "place": "_1", "rvalue": {"kind": "param", "index": 0}},
            {"kind": "assign", "place": "_2", "rvalue": {"kind": "param", "index": 1}},
            {"kind": "assign", "place": "_3", "rvalue": {"kind": "call", "func": "array_len", "args": ["_1"]}},
            ...
          ],
          "terminator": {"kind": "goto", "target": "bb1"}
        },
        ...
      ],
      "locals": [
        {"name": "_1", "type": "Array<T>"},
        {"name": "_2", "type": "Fn(T) -> U>"},
        ...
      ]
    }
  }
}
```

## Compilation Pipeline Changes

### New Compiler Flags

```bash
# Compile to object file + ABI (default for stdlib)
aether compile --emit=obj,abi module.aether -o module

# Compile to object file only (for final linking)
aether compile --emit=obj module.aether -o module.o

# Compile using pre-compiled dependencies
aether compile --abi-path=stdlib/ --link=stdlib/*.o main.aether -o main

# Generate ABI without compiling (for documentation/tooling)
aether compile --emit=abi-only module.aether -o module.abi
```

### Compilation Phases

```
Phase 1: Parse
  - If .aether file: parse source
  - If importing: read .abi file instead of parsing

Phase 2: Semantic Analysis
  - Build symbol table from:
    - Parsed AST (current module)
    - ABI files (dependencies)
  - Type check
  - Resolve imports to ABI symbols

Phase 3: MIR Lowering
  - Lower AST to MIR
  - For generic functions: store MIR for later monomorphization
  - For monomorphization: read MIR from dependency .abi/.mir

Phase 4: Verification (optional)
  - Verify contracts
  - Can use axioms from dependency ABIs

Phase 5: Code Generation
  - Generate LLVM IR
  - Output .o file

Phase 6: ABI Generation
  - Extract public signatures
  - Serialize to .abi file
  - Optionally embed MIR for generics

Phase 7: Linking
  - Collect all .o files
  - Link with system libraries
  - Output executable
```

## Module Loader Changes

### Current Flow
```
import std.io as io
    │
    ▼
Read stdlib/io.aether
    │
    ▼
Parse to AST
    │
    ▼
Extract function signatures
    │
    ▼
Add to symbol table
```

### New Flow
```
import std.io as io
    │
    ▼
Check for stdlib/io.abi
    │
    ├── Found ──▶ Read ABI JSON ──▶ Add to symbol table
    │
    └── Not found ──▶ Fall back to parsing .aether (dev mode)
```

### Module Resolution Order

1. Check `--abi-path` directories for `module.abi`
2. Check standard locations (`~/.aether/lib/`, `/usr/lib/aether/`)
3. Check source directories for `module.aether` (fallback)

## Build System

### Stdlib Makefile

```makefile
AETHER := ./target/release/aether-compiler
STDLIB_SRC := $(wildcard stdlib/*.aether)
STDLIB_OBJ := $(STDLIB_SRC:.aether=.o)
STDLIB_ABI := $(STDLIB_SRC:.aether=.abi)

# Build order matters for dependencies
MODULES := core io string stringview math collections

.PHONY: stdlib
stdlib: $(foreach m,$(MODULES),stdlib/$(m).o stdlib/$(m).abi)

stdlib/core.o stdlib/core.abi: stdlib/core.aether
	$(AETHER) compile --emit=obj,abi $< -o stdlib/core

stdlib/io.o stdlib/io.abi: stdlib/io.aether stdlib/core.abi
	$(AETHER) compile --emit=obj,abi --abi-path=stdlib/ $< -o stdlib/io

stdlib/string.o stdlib/string.abi: stdlib/string.aether stdlib/core.abi
	$(AETHER) compile --emit=obj,abi --abi-path=stdlib/ $< -o stdlib/string

# ... etc

.PHONY: clean
clean:
	rm -f stdlib/*.o stdlib/*.abi
```

### User Project Compilation

```bash
# Simple case: compile with stdlib
aether compile --stdlib main.aether -o main

# Explicit: specify ABI path and link objects
aether compile \
  --abi-path=~/.aether/lib/std/ \
  --link=~/.aether/lib/std/*.o \
  main.aether -o main

# Development: use source stdlib (slower, but allows debugging)
aether compile --stdlib-source main.aether -o main
```

## Type Serialization

### Type Representation in ABI

```json
{
  "type_syntax": {
    "primitives": ["Int", "Int64", "Float", "Bool", "String", "Void"],

    "array": {"element": "<type>"},
    "tuple": {"elements": ["<type>", ...]},
    "function": {"params": ["<type>", ...], "return": "<type>"},
    "reference": {"target": "<type>", "mutable": false},
    "pointer": {"target": "<type>", "mutable": false},

    "generic_instance": {"base": "Array", "args": ["Int"]},
    "generic_param": {"name": "T"},

    "user_defined": {"module": "std.collections", "name": "HashMap"}
  }
}
```

### Examples

```json
// Int
{"kind": "primitive", "name": "Int"}

// Array<String>
{"kind": "generic_instance", "base": "Array", "args": [
  {"kind": "primitive", "name": "String"}
]}

// Fn(Int, Int) -> Bool
{"kind": "function", "params": [
  {"kind": "primitive", "name": "Int"},
  {"kind": "primitive", "name": "Int"}
], "return": {"kind": "primitive", "name": "Bool"}}

// &mut T (generic borrowed reference)
{"kind": "reference", "mutable": true, "target":
  {"kind": "generic_param", "name": "T"}
}

// HashMap<String, Vec<Int>>
{"kind": "generic_instance", "base": "HashMap", "args": [
  {"kind": "primitive", "name": "String"},
  {"kind": "generic_instance", "base": "Vec", "args": [
    {"kind": "primitive", "name": "Int"}
  ]}
]}
```

## Verification in Separate Compilation

### Contract Storage

Contracts are stored in the ABI for two purposes:
1. **Documentation** - Tools can display contracts
2. **Verification** - Calling code can verify against contracts

### Verification Modes

```bash
# Verify contracts at call sites using ABI contracts
aether compile --verify=caller main.aether

# Trust ABI contracts (skip verification)
aether compile --verify=none main.aether

# Re-verify everything including stdlib (slow)
aether compile --verify=full --stdlib-source main.aether
```

### Axiom Propagation

When verifying generic code, axioms from trait definitions propagate:

```json
// In std.collections.abi
{
  "functions": [{
    "name": "sort",
    "signature": {
      "generic_params": [{"name": "T", "kind": "type"}],
      "where_clauses": [{"type": "T", "trait": "Ord"}],
      ...
    },
    "contracts": {
      "postconditions": [
        {"expr": "is_sorted(result)", "message": "Output is sorted"}
      ],
      "assumes_axioms": ["Ord.transitivity", "Ord.totality"]
    }
  }]
}
```

## Error Messages

ABI files include source locations for good error messages:

```
error[E0308]: mismatched types
  --> main.aether:15:20
   |
15 |     let x = sv.view_length(42);
   |                            ^^ expected Int64, found Int
   |
note: function defined here
  --> stdlib/stringview.aether:36:1 (from stdlib/stringview.abi)
   |
36 | func view_length(view: Int64) -> Int;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## Incremental Compilation (Future)

### Dependency Graph

```json
// .aether-cache/deps.json
{
  "modules": {
    "main": {
      "source_hash": "sha256:abc...",
      "abi_hash": null,
      "depends_on": ["std.io", "std.stringview"],
      "last_compiled": "2024-01-15T10:30:00Z"
    },
    "std.io": {
      "source_hash": "sha256:def...",
      "abi_hash": "sha256:ghi...",
      "depends_on": ["std.core"],
      "last_compiled": "2024-01-10T08:00:00Z"
    }
  }
}
```

### Recompilation Rules

1. If source hash changed → recompile
2. If any dependency ABI hash changed → recompile
3. If compiler version changed → recompile all
4. Otherwise → use cached .o

## Implementation Tasks

### Task 1: ABI Data Structures
- [ ] Define `AbiModule` struct in Rust
- [ ] Define serialization/deserialization (serde)
- [ ] Add ABI version checking

### Task 2: ABI Generation
- [ ] Extract public symbols after semantic analysis
- [ ] Serialize types to JSON representation
- [ ] Write .abi file alongside .o file
- [ ] Add `--emit=abi` flag

### Task 3: ABI Loading
- [ ] Read .abi files in module loader
- [ ] Convert ABI types to internal Type representation
- [ ] Build symbol table from ABI
- [ ] Skip parsing .aether when .abi available

### Task 4: MIR Storage for Generics
- [ ] Serialize MIR to JSON or binary format
- [ ] Store in .abi or separate .mir file
- [ ] Load MIR during monomorphization

### Task 5: Linking Multiple Objects
- [ ] Collect all required .o files
- [ ] Pass to linker (clang/ld)
- [ ] Handle symbol visibility

### Task 6: Build System
- [ ] Create stdlib Makefile
- [ ] Add `--abi-path` flag
- [ ] Add `--link` flag for object files
- [ ] Default stdlib compilation

### Task 7: Testing
- [ ] Unit tests for ABI serialization
- [ ] Integration tests for separate compilation
- [ ] Test generic monomorphization across modules
- [ ] Test contract verification with ABIs

## Migration Path

### Phase 1: Basic Separate Compilation
- Non-generic functions only
- JSON ABI format
- Manual stdlib build

### Phase 2: Generic Support
- MIR storage and loading
- Monomorphization across modules
- Trait method resolution

### Phase 3: Optimization
- Binary ABI format (optional)
- Incremental compilation
- Parallel compilation

### Phase 4: Distribution
- Package format (.aetherpkg)
- Package manager integration
- Version resolution

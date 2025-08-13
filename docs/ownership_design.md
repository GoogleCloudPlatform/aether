# AetherScript Ownership System Design

## Overview

This document outlines the architectural changes needed to implement AetherScript's ownership system, which provides memory safety without garbage collection through compile-time ownership tracking.

## Current State

The ownership tokens (`^T`, `&T`, `&mut T`, `~T`) are already:
- Recognized by the lexer as tokens
- Parsed into AST type specifiers
- Have placeholder semantic analysis

However, the Type system doesn't include ownership information, making it impossible to track ownership transfers and enforce borrowing rules.

## Required Changes

### 1. Type System Extension

#### Current Type Enum
```rust
pub enum Type {
    Primitive(PrimitiveType),
    Array { element_type: Box<Type>, size: Option<usize> },
    Map { key_type: Box<Type>, value_type: Box<Type> },
    Named { name: String, type_params: Vec<Type> },
    Function { params: Vec<Type>, return_type: Box<Type> },
    TypeVariable(String),
    // ... other variants
}
```

#### Proposed Type Enum
```rust
pub enum Type {
    Primitive(PrimitiveType),
    Array { element_type: Box<Type>, size: Option<usize> },
    Map { key_type: Box<Type>, value_type: Box<Type> },
    Named { name: String, type_params: Vec<Type> },
    Function { params: Vec<Type>, return_type: Box<Type> },
    TypeVariable(String),
    // New: Ownership wrapper
    Owned { 
        ownership: OwnershipKind,
        base_type: Box<Type>,
    },
    // ... other variants
}

pub enum OwnershipKind {
    Owned,           // ^T - Single owner
    Borrowed,        // &T - Immutable borrow
    MutableBorrow,   // &mut T - Mutable borrow
    Shared,          // ~T - Reference counted
}
```

### 2. Symbol Table Enhancement

The symbol table already has move/borrow tracking functions that are never called:
- `record_move(symbol: &str, location: SourceLocation)`
- `record_borrow(symbol: &str, is_mutable: bool, location: SourceLocation)`
- `check_moved(symbol: &str) -> Option<&SourceLocation>`
- `is_borrowed(symbol: &str) -> bool`

These need to be integrated into the semantic analyzer.

### 3. Semantic Analysis Integration

#### Function Calls
When analyzing function calls, track ownership transfers:

```rust
// In analyze_function_call
for (i, arg) in call.arguments.iter().enumerate() {
    let arg_type = self.analyze_expression(arg)?;
    
    // Check ownership transfer
    if let Some(param_type) = param_types.get(i) {
        match (extract_ownership(&arg_type), extract_ownership(param_type)) {
            (Some(OwnershipKind::Owned), Some(OwnershipKind::Owned)) => {
                // Ownership transfer - record move
                if let Expression::Variable { name, .. } = &arg.value {
                    self.symbol_table.record_move(&name.name, arg.source_location.clone())?;
                }
            }
            (Some(OwnershipKind::Owned), Some(OwnershipKind::Borrowed)) => {
                // Borrowing - record borrow
                if let Expression::Variable { name, .. } = &arg.value {
                    self.symbol_table.record_borrow(&name.name, false, arg.source_location.clone())?;
                }
            }
            // ... other cases
        }
    }
}
```

#### Variable Usage
Check if variables have been moved before use:

```rust
// In analyze_variable
if let Some(moved_location) = self.symbol_table.check_moved(&name.name) {
    return Err(SemanticError::UseAfterMove {
        variable: name.name.clone(),
        moved_at: moved_location.clone(),
        used_at: source_location.clone(),
    });
}
```

### 4. MIR Changes

The MIR needs to track ownership in types and handle cleanup:

```rust
// In mir/types.rs
impl Type {
    pub fn owned(base: Type) -> Self {
        Type::Owned {
            ownership: OwnershipKind::Owned,
            base_type: Box::new(base),
        }
    }
    
    pub fn borrowed(base: Type) -> Self {
        Type::Owned {
            ownership: OwnershipKind::Borrowed,
            base_type: Box::new(base),
        }
    }
}
```

### 5. LLVM Backend Changes

The LLVM backend needs to:
1. Handle cleanup for owned values going out of scope
2. Ensure proper reference counting for shared values
3. Track lifetimes for borrowed values

```rust
// In generate_terminator for Return
if let Some(cleanup_block) = function.cleanup_blocks.get(&current_block) {
    // Generate cleanup code for owned values
    for local in &cleanup_block.locals_to_drop {
        generate_drop(local, builder)?;
    }
}
```

## Implementation Order

1. **Phase 1: Type System** ✓
   - Extend Type enum with ownership information
   - Update type conversion in AST to Type mapping
   - Add ownership extraction utilities

2. **Phase 2: Basic Tracking**
   - Integrate move/borrow tracking in semantic analyzer
   - Add use-after-move detection
   - Implement basic borrow checking

3. **Phase 3: Function Calls**
   - Track ownership transfers in function calls
   - Validate ownership compatibility
   - Handle return value ownership

4. **Phase 4: Cleanup & Drops**
   - Track owned values that need cleanup
   - Generate drop calls in LLVM backend
   - Handle early returns and error paths

5. **Phase 5: Advanced Features**
   - Lifetime inference
   - Borrow splitting
   - Interior mutability patterns

## Testing Strategy

1. **Unit Tests**
   - Type system ownership representation
   - Symbol table move/borrow tracking
   - Ownership compatibility checks

2. **Integration Tests**
   - Basic ownership transfer
   - Use-after-move detection
   - Borrow checking scenarios
   - Memory leak prevention

3. **Example Programs**
   - Resource management patterns
   - Producer-consumer with ownership
   - Tree traversal with borrows

## Error Messages

New error types needed:
- `UseAfterMove`: Variable used after ownership transferred
- `CannotMoveWhileBorrowed`: Cannot move value that is borrowed
- `MultipleImmutableBorrows`: Cannot borrow mutably while immutably borrowed
- `OwnershipMismatch`: Function expects different ownership than provided

## Future Considerations

1. **Lifetime Parameters**: Generic lifetime annotations for complex borrowing
2. **Async Ownership**: Ownership across async boundaries
3. **Unsafe Blocks**: Escape hatch for low-level operations
4. **Custom Drop**: User-defined cleanup logic
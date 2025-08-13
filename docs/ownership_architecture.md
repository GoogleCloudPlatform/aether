# Ownership System Architecture Design

## Overview

This document outlines the architectural changes required to fully implement AetherScript's ownership system. The goal is to provide compile-time memory safety guarantees through ownership tracking, borrowing rules, and move semantics.

## Current Architecture

### AST Layer (✅ Complete)
- `TypeSpecifier::Owned` variant stores ownership annotations from source
- `OwnershipKind` enum defines ownership types: Owned (^T), Borrowed (&T), BorrowedMut (&mut T), Shared (~T)
- Parser correctly builds ownership-annotated type specifications

### Symbol Table (✅ Complete)
- Tracks `is_moved: bool` for each symbol
- Tracks `borrow_state: BorrowState` (None, Borrowed, BorrowedMut)
- Provides methods: `mark_variable_moved()`, `borrow_variable()`, `borrow_variable_mut()`

### Type System (❌ Incomplete)
- `Type` enum has no ownership information
- Type conversion from `TypeSpecifier` to `Type` loses ownership annotations
- Function types store parameter types without ownership

## Proposed Architecture Changes

### 1. Extended Type System

```rust
// Modify src/types/mod.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // ... existing variants ...
    
    /// Ownership-annotated type
    Owned {
        base_type: Box<Type>,
        ownership: OwnershipKind,
    },
}

// Add ownership helper methods
impl Type {
    /// Check if this type transfers ownership
    pub fn transfers_ownership(&self) -> bool {
        matches!(self, Type::Owned { 
            ownership: OwnershipKind::Owned, .. 
        })
    }
    
    /// Check if this type is a borrowed reference
    pub fn is_borrowed_ref(&self) -> bool {
        matches!(self, Type::Owned { 
            ownership: OwnershipKind::Borrowed, .. 
        })
    }
    
    /// Check if this type is a mutable borrowed reference
    pub fn is_borrowed_mut_ref(&self) -> bool {
        matches!(self, Type::Owned { 
            ownership: OwnershipKind::BorrowedMut, .. 
        })
    }
    
    /// Strip ownership annotation to get base type
    pub fn base_type(&self) -> &Type {
        match self {
            Type::Owned { base_type, .. } => base_type,
            _ => self,
        }
    }
}
```

### 2. Type Conversion Updates

```rust
// In type_from_specifier() or equivalent

fn convert_type_specifier(spec: &TypeSpecifier) -> Result<Type, Error> {
    match spec {
        TypeSpecifier::Owned { base_type, ownership, .. } => {
            let base = convert_type_specifier(base_type)?;
            Ok(Type::Owned {
                base_type: Box::new(base),
                ownership: *ownership,
            })
        }
        // ... other variants ...
    }
}
```

### 3. Function Type Enhancement

Option A: Embed ownership in parameter types
```rust
// Function types naturally preserve ownership through Type::Owned
Type::Function {
    parameter_types: vec![
        Type::Owned { 
            base_type: Box::new(Type::Primitive(PrimitiveType::Integer)),
            ownership: OwnershipKind::Owned,
        }
    ],
    return_type: Box::new(Type::Primitive(PrimitiveType::Void)),
}
```

Option B: Separate ownership tracking
```rust
pub struct FunctionTypeInfo {
    pub parameter_types: Vec<Type>,
    pub parameter_ownership: Vec<Option<OwnershipKind>>,
    pub return_type: Type,
    pub return_ownership: Option<OwnershipKind>,
}
```

**Recommendation**: Option A is cleaner and maintains consistency.

### 4. Semantic Analysis Updates

```rust
// In analyze_function_call()

for (arg, param_type) in call.arguments.iter().zip(parameter_types.iter()) {
    match param_type {
        Type::Owned { ownership: OwnershipKind::Owned, .. } => {
            // Transfer ownership
            if let Expression::Variable { name, .. } = arg.value.as_ref() {
                self.symbol_table.mark_variable_moved(&name.name)?;
            }
        }
        Type::Owned { ownership: OwnershipKind::Borrowed, .. } => {
            // Immutable borrow
            if let Expression::Variable { name, .. } = arg.value.as_ref() {
                self.symbol_table.borrow_variable(&name.name)?;
            }
        }
        Type::Owned { ownership: OwnershipKind::BorrowedMut, .. } => {
            // Mutable borrow
            if let Expression::Variable { name, .. } = arg.value.as_ref() {
                self.symbol_table.borrow_variable_mut(&name.name)?;
            }
        }
        _ => {} // Non-owned types
    }
}
```

### 5. Reference Creation Handling

```rust
// In analyze_expression() for AddressOf

Expression::AddressOf { operand, .. } => {
    let operand_type = self.analyze_expression(operand)?;
    
    // Determine ownership based on context
    let ownership = if self.expecting_mutable_ref() {
        // Update borrow state
        if let Expression::Variable { name, .. } = operand {
            self.symbol_table.borrow_variable_mut(&name.name)?;
        }
        OwnershipKind::BorrowedMut
    } else {
        // Update borrow state
        if let Expression::Variable { name, .. } = operand {
            self.symbol_table.borrow_variable(&name.name)?;
        }
        OwnershipKind::Borrowed
    };
    
    Ok(Type::Owned {
        base_type: Box::new(operand_type),
        ownership,
    })
}
```

### 6. Borrow Scope Management

```rust
pub struct BorrowScope {
    borrows: Vec<BorrowInfo>,
}

pub struct BorrowInfo {
    variable: String,
    borrow_type: BorrowState,
    scope_depth: usize,
}

impl SemanticAnalyzer {
    fn enter_borrow_scope(&mut self) {
        self.borrow_scopes.push(BorrowScope::new());
    }
    
    fn exit_borrow_scope(&mut self) {
        if let Some(scope) = self.borrow_scopes.pop() {
            // Release all borrows in this scope
            for borrow in scope.borrows {
                self.symbol_table.release_borrow(&borrow.variable);
            }
        }
    }
}
```

## Implementation Plan

### Phase 1: Type System Extension (High Priority)
1. Add `Type::Owned` variant
2. Update type conversion to preserve ownership
3. Add ownership query methods to Type
4. Update type equality/compatibility checks

### Phase 2: Function Call Ownership (High Priority)
1. Activate ownership checking in `analyze_function_call()`
2. Implement move detection for owned parameters
3. Implement borrow tracking for reference parameters
4. Add tests for ownership transfer scenarios

### Phase 3: Reference Management (Medium Priority)
1. Update `AddressOf` expression handling
2. Track active borrows in semantic analyzer
3. Implement borrow scope management
4. Validate borrowing rules (no mut while borrowed, etc.)

### Phase 4: Advanced Features (Low Priority)
1. Lifetime analysis for complex borrowing patterns
2. Shared ownership (~T) with reference counting
3. Ownership inference for better ergonomics
4. Integration with error recovery

## Testing Strategy

### Unit Tests
- Type system preserves ownership through conversions
- Symbol table correctly tracks moves and borrows
- Function calls properly transfer ownership

### Integration Tests
- Use-after-move detection
- Multiple immutable borrows allowed
- Mutable borrow exclusivity
- Borrow scope correctness

### Example Test Cases

```aether
; Should fail: use after move
(let ((x 42))
    (take_ownership x)  ; x moved here
    (print x))          ; ERROR: use after move

; Should pass: multiple immutable borrows
(let ((x 42))
    (let ((ref1 &x)
          (ref2 &x))    ; OK: multiple immutable borrows
        (+ ref1 ref2)))

; Should fail: mutable borrow while borrowed
(let ((mut x 42))
    (let ((ref &x))
        (let ((mut_ref &mut x))  ; ERROR: x already borrowed
            ...)))
```

## Performance Considerations

- Ownership checking adds minimal overhead during compilation
- No runtime cost - all checks are compile-time
- Symbol table lookups are O(1) with HashMap
- Borrow tracking requires stack-based scope management

## Migration Strategy

1. Implement Type::Owned without breaking existing code
2. Add ownership gradually to standard library functions
3. Provide clear error messages for ownership violations
4. Document migration guide for existing code

## Open Questions

1. Should we infer ownership annotations in some cases?
2. How to handle ownership in generic types?
3. Should we support explicit lifetime annotations?
4. How to integrate with FFI ownership semantics?

## Conclusion

The proposed architecture extends the existing type system to include ownership information while maintaining backward compatibility. The implementation can be done incrementally, with the most critical features (type system extension and function call ownership) implemented first.
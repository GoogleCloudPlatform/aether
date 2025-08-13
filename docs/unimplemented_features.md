# Unimplemented Language Features in AetherScript

This document lists language features that have keywords defined in the lexer but are not fully implemented across the compiler pipeline.

## 1. Ownership and Memory Management Keywords

### Keywords in Lexer but Not Implemented:
- **OWNERSHIP** - Defined in lexer, has AST support (OwnershipInfo), but no parser implementation
- **LIFETIME** - Defined in lexer, has AST support (Lifetime enum), but no parser implementation  
- **PASSING** - Defined in lexer, has AST support (PassingMode), but no parser implementation
- **BY_VALUE** - Defined in lexer, maps to PassingMode::ByValue, but no parser implementation
- **BY_REFERENCE** - Defined in lexer, maps to PassingMode::ByReference, but no parser implementation

### Status:
- AST has full support for ownership/lifetime concepts
- Parser has KeywordType enum entries but no parsing logic
- No semantic analysis for ownership tracking
- No MIR lowering for ownership semantics
- No LLVM codegen for ownership

## 2. Metadata/Contract Keywords  

### Keywords in Lexer but Partially Implemented:
- **PRECONDITION** - Defined in lexer, AST support exists, parser has keyword but no parsing logic
- **POSTCONDITION** - Defined in lexer, AST support exists, parser has keyword but no parsing logic
- **INVARIANT** - Defined in lexer, AST support exists, parser has keyword but no parsing logic
- **ALGORITHM_HINT** - Defined in lexer, AST support exists, parser has keyword but no parsing logic
- **PERFORMANCE_EXPECTATION** - Defined in lexer, AST support exists, parser has keyword but no parsing logic
- **COMPLEXITY_EXPECTATION** - Defined in lexer, AST support exists, parser has keyword but no parsing logic

### Status:
- AST has FunctionMetadata structure with fields for all these
- Parser recognizes keywords but doesn't parse them in function definitions
- No semantic analysis for contracts
- No verification/checking of contracts

## 3. FFI Keywords

### Keywords in Lexer and Partially Implemented:
- **CALLING_CONVENTION** - Implemented in external function parsing
- **THREAD_SAFE** - Implemented in external function parsing  
- **MAY_BLOCK** - Implemented in external function parsing

### Status:
- External function declarations support these keywords
- Regular functions don't parse these metadata fields
- No propagation to LLVM backend

## 4. Pointer Operations

### Missing from Lexer Entirely:
- **ADDRESS_OF** - AST has AddressOf expression variant, but no lexer keyword
- **DEREFERENCE** - AST has Dereference expression variant, but no lexer keyword  
- **POINTER_ADD** - No AST support, no lexer keyword

### Status:
- AST supports AddressOf and Dereference expressions
- No lexer keywords defined
- No parser implementation
- No semantic analysis for pointer operations
- No MIR/LLVM support

## 5. Map Operations

### Keywords in Lexer but Not Fully Implemented:
- **MAP_FROM_TYPE_TO_TYPE** - Implemented in type parser
- **MAP_LITERAL** - Defined in lexer, AST has MapLiteral expression, but no parser implementation
- **GET_MAP_VALUE** - Defined in lexer, parser has keyword but no implementation
- **SET_MAP_VALUE** - Defined in lexer, parser has keyword but no implementation

### Status:
- Type system supports Map types
- MapLiteral expression exists in AST but can't be parsed
- Map access operations not implemented in parser
- No runtime support for maps

## 6. Mutability Keywords

### Keywords in Lexer and Partially Implemented:
- **MUTABILITY** - Defined in lexer, parser has keyword
- **MUTABLE** - Defined in lexer, parser has keyword  
- **IMMUTABLE** - Defined in lexer, parser has keyword

### Status:
- AST has Mutability enum
- Variable declarations support mutability
- No enforcement in semantic analysis
- No propagation through type system

## Summary of Implementation Gaps

1. **Ownership System**: Keywords exist but no implementation at any level
2. **Contract/Metadata System**: AST support exists but no parsing or verification
3. **Pointer Operations**: AST support but missing lexer keywords and parser
4. **Map Literals**: Partial implementation - types work but literals don't
5. **Mutability**: Partial - declarations work but not enforced

## Recommended Implementation Priority

1. **Pointer Operations**: Add ADDRESS_OF and DEREFERENCE keywords to lexer, implement parsing
2. **Map Operations**: Implement MAP_LITERAL parsing and map access operations
3. **Mutability Enforcement**: Add semantic analysis for mutability checking
4. **Contract Parsing**: Implement parsing for function metadata/contracts
5. **Ownership System**: Design and implement full ownership tracking
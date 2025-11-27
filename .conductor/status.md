# Aether V2 Syntax Migration Status

## Phase 4: Example Verification (Active)

- [x] 01-basics
- [x] 02-variables
- [x] 03-types
- [x] 04-functions
- [x] 05-operators
- [x] 06-control-flow
- [x] 07-structs
- [x] 08-enums
- [x] 09-pattern-matching
- [ ] 10-collections
    - [x] arrays (partial)
    - [x] maps
- [ ] 11-memory
- [ ] 12-error-handling
- [ ] 13-strings
    - [x] string_basics
    - [x] string_operations
- [ ] 14-ffi
- [ ] 15-stdlib
- [ ] 16-networking

## Tasks

- [x] Fix `parse_map_literal` and `looks_like_map_literal`.
- [x] Implement `lower_method_call` for `map.insert` and `map.get`.
- [x] Implement semantic analysis for `Map` method calls.
- [x] Verify `maps` example.
- [x] Verify `string_basics` example.
- [x] Verify `string_operations` example.
- [ ] Verify `arrays` example.
- [ ] Verify remaining examples.

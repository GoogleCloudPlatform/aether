# AetherScript Technical Documentation

This directory contains detailed technical documentation for AetherScript developers and contributors.

## Architecture and Design

### Core Architecture
- **[archAetherScript.md](archAetherScript.md)** - Complete compiler architecture and design
- **[FINAL_DESIGN.md](../FINAL_DESIGN.md)** - Core design philosophy and principles (in root)

### Ownership System
- **[ownership_architecture.md](ownership_architecture.md)** - Ownership system architectural design
- **[ownership_design.md](ownership_design.md)** - Implementation details and semantics
- **[mutability_and_borrowing.md](mutability_and_borrowing.md)** - Mutability rules and borrow checking

### Language Features
- **[enhanced_verification.md](enhanced_verification.md)** - Contract verification and formal methods
- **[resource_management.md](resource_management.md)** - Resource management and RAII patterns
- **[llm_optimized_errors.md](llm_optimized_errors.md)** - LLM-friendly error reporting system
- **[error_handling.md](error_handling.md)** - Runtime error handling and panic subsystem

## Implementation Details

### Standard Library
- **[standard_library.md](standard_library.md)** - Standard library design and modules
- **[function_metadata.md](function_metadata.md)** - Function metadata and introspection

### FFI and Runtime
- **[test_ffi_callbacks_summary.md](test_ffi_callbacks_summary.md)** - FFI callback implementation
- **[http_server_chat.md](http_server_chat.md)** - HTTP server implementation example

## Development Process

### Testing and Quality
- **[TESTING_PLAN.md](TESTING_PLAN.md)** - Comprehensive testing strategy
- **[missing_features.md](missing_features.md)** - Tracking incomplete features

### Development Roadmap  
- **[NEXT_STEPS.md](NEXT_STEPS.md)** - Development roadmap and future plans
- **[unimplemented_features.md](unimplemented_features.md)** - Detailed unimplemented features list

### User Guides
- **[running_examples.md](running_examples.md)** - How to run and test examples

## Organization

This documentation is organized by:
1. **Architecture** - High-level design and system architecture
2. **Implementation** - Detailed implementation specifications  
3. **Development** - Process, testing, and roadmap documentation

For user-facing documentation, see the main project root:
- [User Guide](../user-guide.md) - Complete language tutorial
- [Language Reference](../LANGUAGE_REFERENCE.md) - Syntax and semantics reference
- [Examples](../examples/README.md) - Working code examples

## Contributing to Documentation

When adding new technical documentation:
1. Place user-facing docs in the project root
2. Place technical/implementation docs in this `docs/` folder
3. Update this README with new document descriptions
4. Follow existing documentation style and format
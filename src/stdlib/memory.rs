//! std.memory - Memory management and allocation module

use crate::ast::{Module, TypeSpecifier, PrimitiveType, Identifier, ExportStatement};
use crate::error::SourceLocation;
use crate::ast::CallingConvention;
use super::{create_external_function_named, create_function_stub};
use std::collections::HashMap;

/// Create the std.memory module with allocation operations
pub fn create_memory_module() -> Module {
    let mut external_functions = HashMap::new();
    let mut functions = HashMap::new();
    
    // Common types
    let size_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::SizeT,
        source_location: SourceLocation::unknown(),
    };
    let void_ptr_type = TypeSpecifier::Pointer {
        target_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: SourceLocation::unknown(),
        }),
        is_mutable: true,
        source_location: SourceLocation::unknown(),
    };
    let bool_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Boolean,
        source_location: SourceLocation::unknown(),
    };
    let int_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Integer,
        source_location: SourceLocation::unknown(),
    };
    
    // Memory allocation functions (external for performance)
    external_functions.insert("allocate".to_string(), create_external_function_named(
        "allocate",
        "aether_memory_allocate",
        vec![("size", size_type.clone())],
        void_ptr_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("allocate_zeroed".to_string(), create_external_function_named(
        "allocate_zeroed",
        "aether_memory_allocate_zeroed",
        vec![("size", size_type.clone())],
        void_ptr_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("reallocate".to_string(), create_external_function_named(
        "reallocate",
        "aether_memory_reallocate",
        vec![
            ("ptr", void_ptr_type.clone()),
            ("new_size", size_type.clone()),
        ],
        void_ptr_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("deallocate".to_string(), create_external_function_named(
        "deallocate",
        "aether_memory_deallocate",
        vec![("ptr", void_ptr_type.clone())],
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: SourceLocation::unknown(),
        },
        CallingConvention::C,
    ));
    
    external_functions.insert("copy_memory".to_string(), create_external_function_named(
        "copy_memory",
        "aether_memory_copy",
        vec![
            ("dest", void_ptr_type.clone()),
            ("src", void_ptr_type.clone()),
            ("size", size_type.clone()),
        ],
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: SourceLocation::unknown(),
        },
        CallingConvention::C,
    ));
    
    external_functions.insert("move_memory".to_string(), create_external_function_named(
        "move_memory",
        "aether_memory_move",
        vec![
            ("dest", void_ptr_type.clone()),
            ("src", void_ptr_type.clone()),
            ("size", size_type.clone()),
        ],
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: SourceLocation::unknown(),
        },
        CallingConvention::C,
    ));
    
    external_functions.insert("set_memory".to_string(), create_external_function_named(
        "set_memory",
        "aether_memory_set",
        vec![
            ("ptr", void_ptr_type.clone()),
            ("value", int_type.clone()),
            ("size", size_type.clone()),
        ],
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: SourceLocation::unknown(),
        },
        CallingConvention::C,
    ));
    
    external_functions.insert("compare_memory".to_string(), create_external_function_named(
        "compare_memory",
        "aether_memory_compare",
        vec![
            ("ptr1", void_ptr_type.clone()),
            ("ptr2", void_ptr_type.clone()),
            ("size", size_type.clone()),
        ],
        int_type.clone(),
        CallingConvention::C,
    ));
    
    // Memory alignment functions
    external_functions.insert("allocate_aligned".to_string(), create_external_function_named(
        "allocate_aligned",
        "aether_memory_allocate_aligned",
        vec![
            ("size", size_type.clone()),
            ("alignment", size_type.clone()),
        ],
        void_ptr_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("is_aligned".to_string(), create_external_function_named(
        "is_aligned",
        "aether_memory_is_aligned",
        vec![
            ("ptr", void_ptr_type.clone()),
            ("alignment", size_type.clone()),
        ],
        bool_type.clone(),
        CallingConvention::C,
    ));
    
    // Memory statistics
    external_functions.insert("get_allocation_size".to_string(), create_external_function_named(
        "get_allocation_size",
        "aether_memory_get_allocation_size",
        vec![("ptr", void_ptr_type.clone())],
        size_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("get_total_allocated".to_string(), create_external_function_named(
        "get_total_allocated",
        "aether_memory_get_total_allocated",
        vec![],
        size_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("get_peak_allocated".to_string(), create_external_function_named(
        "get_peak_allocated",
        "aether_memory_get_peak_allocated",
        vec![],
        size_type.clone(),
        CallingConvention::C,
    ));
    
    // High-level memory management functions (implemented in AetherScript)
    functions.insert("create_buffer".to_string(), create_function_stub(
        "create_buffer",
        vec![
            ("size", size_type.clone()),
            ("zero_init", bool_type.clone()),
        ],
        void_ptr_type.clone(),
    ));
    
    functions.insert("resize_buffer".to_string(), create_function_stub(
        "resize_buffer",
        vec![
            ("buffer", void_ptr_type.clone()),
            ("new_size", size_type.clone()),
        ],
        void_ptr_type.clone(),
    ));
    
    functions.insert("destroy_buffer".to_string(), create_function_stub(
        "destroy_buffer",
        vec![("buffer", void_ptr_type.clone())],
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: SourceLocation::unknown(),
        },
    ));
    
    functions.insert("duplicate_buffer".to_string(), create_function_stub(
        "duplicate_buffer",
        vec![
            ("buffer", void_ptr_type.clone()),
            ("size", size_type.clone()),
        ],
        void_ptr_type.clone(),
    ));
    
    // Memory pool management
    functions.insert("create_memory_pool".to_string(), create_function_stub(
        "create_memory_pool",
        vec![
            ("block_size", size_type.clone()),
            ("num_blocks", size_type.clone()),
        ],
        void_ptr_type.clone(), // Pool handle
    ));
    
    functions.insert("pool_allocate".to_string(), create_function_stub(
        "pool_allocate",
        vec![("pool", void_ptr_type.clone())],
        void_ptr_type.clone(),
    ));
    
    functions.insert("pool_deallocate".to_string(), create_function_stub(
        "pool_deallocate",
        vec![
            ("pool", void_ptr_type.clone()),
            ("ptr", void_ptr_type.clone()),
        ],
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: SourceLocation::unknown(),
        },
    ));
    
    functions.insert("destroy_memory_pool".to_string(), create_function_stub(
        "destroy_memory_pool",
        vec![("pool", void_ptr_type.clone())],
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: SourceLocation::unknown(),
        },
    ));
    
    Module {
        name: Identifier::new("std.memory".to_string(), SourceLocation::unknown()),
        intent: Some("Provides low-level memory management operations and allocation utilities".to_string()),
        imports: vec![],
        exports: vec![
            ExportStatement::Function {
                name: Identifier::new("allocate".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("deallocate".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("create_buffer".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("create_memory_pool".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
        ],
        type_definitions: vec![],
        constant_declarations: vec![],
        function_definitions: functions.into_values().collect(),
        external_functions: external_functions.into_values().collect(),
        source_location: SourceLocation::unknown(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_module_creation() {
        let module = create_memory_module();
        
        assert_eq!(module.name.name, "std.memory");
        assert!(module.intent.is_some());
        
        // Check basic allocation operations
        assert!(module.external_functions.iter().any(|f| f.name.name == "allocate"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "deallocate"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "reallocate"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "copy_memory"));
        
        // Check high-level functions
        assert!(module.function_definitions.iter().any(|f| f.name.name == "create_buffer"));
        assert!(module.function_definitions.iter().any(|f| f.name.name == "create_memory_pool"));
        
        // Check exports
        assert!(module.exports.iter().any(|e| matches!(e, ExportStatement::Function { name, .. } if name.name == "allocate")));
    }
    
    #[test]
    fn test_memory_function_signatures() {
        let module = create_memory_module();
        
        // Test allocate function
        let allocate = module.external_functions.iter()
            .find(|f| f.name.name == "allocate")
            .expect("allocate function not found");
        assert_eq!(allocate.parameters.len(), 1);
        assert_eq!(allocate.parameters[0].name.name, "size");
        
        // Test copy_memory function
        let copy_memory = module.external_functions.iter()
            .find(|f| f.name.name == "copy_memory")
            .expect("copy_memory function not found");
        assert_eq!(copy_memory.parameters.len(), 3);
        assert_eq!(copy_memory.parameters[0].name.name, "dest");
        assert_eq!(copy_memory.parameters[1].name.name, "src");
        assert_eq!(copy_memory.parameters[2].name.name, "size");
    }
}
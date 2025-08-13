//! std.collections - Array and map utility functions

use crate::ast::{Module, TypeSpecifier, PrimitiveType, Identifier, ExportStatement};
use crate::error::SourceLocation;
use crate::ast::CallingConvention;
use super::{create_external_function_named, create_function_stub};
use std::collections::HashMap;

/// Create the std.collections module with array and map utilities
pub fn create_collections_module() -> Module {
    let mut external_functions = HashMap::new();
    let mut functions = HashMap::new();
    
    // Common types
    let int_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Integer,
        source_location: SourceLocation::unknown(),
    };
    let bool_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Boolean,
        source_location: SourceLocation::unknown(),
    };
    let size_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::SizeT,
        source_location: SourceLocation::unknown(),
    };
    let _string_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::String,
        source_location: SourceLocation::unknown(),
    };
    
    // Generic array type (would need proper generics implementation)
    let generic_array_type = TypeSpecifier::Array {
        element_type: Box::new(TypeSpecifier::TypeParameter {
            name: Identifier::new("T".to_string(), SourceLocation::unknown()),
            constraints: vec![],
            source_location: SourceLocation::unknown(),
        }),
        size: None, // Dynamic array
        source_location: SourceLocation::unknown(),
    };
    
    // Generic map type
    let generic_map_type = TypeSpecifier::Map {
        key_type: Box::new(TypeSpecifier::TypeParameter {
            name: Identifier::new("K".to_string(), SourceLocation::unknown()),
            constraints: vec![],
            source_location: SourceLocation::unknown(),
        }),
        value_type: Box::new(TypeSpecifier::TypeParameter {
            name: Identifier::new("V".to_string(), SourceLocation::unknown()),
            constraints: vec![],
            source_location: SourceLocation::unknown(),
        }),
        source_location: SourceLocation::unknown(),
    };
    
    // Generic element type
    let generic_element_type = TypeSpecifier::TypeParameter {
        name: Identifier::new("T".to_string(), SourceLocation::unknown()),
        constraints: vec![],
        source_location: SourceLocation::unknown(),
    };
    
    // Array operations (external C implementations for performance)
    external_functions.insert("array_create".to_string(), create_external_function_named(
        "array_create",
        "aether_collections_array_create",
        vec![
            ("initial_capacity", size_type.clone()),
            ("element_size", size_type.clone()),
        ],
        TypeSpecifier::Pointer {
            target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
            is_mutable: true,
            source_location: SourceLocation::unknown(),
        },
        CallingConvention::C,
    ));
    
    external_functions.insert("array_destroy".to_string(), create_external_function_named(
        "array_destroy",
        "aether_collections_array_destroy",
        vec![("array", TypeSpecifier::Pointer {
            target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
            is_mutable: true,
            source_location: SourceLocation::unknown(),
        })],
        TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            },
        CallingConvention::C,
    ));
    
    external_functions.insert("array_length".to_string(), create_external_function_named(
        "array_length",
        "aether_collections_array_length",
        vec![("array", TypeSpecifier::Pointer {
            target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
            is_mutable: false,
            source_location: SourceLocation::unknown(),
        })],
        size_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("array_capacity".to_string(), create_external_function_named(
        "array_capacity",
        "aether_collections_array_capacity",
        vec![("array", TypeSpecifier::Pointer {
            target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
            is_mutable: false,
            source_location: SourceLocation::unknown(),
        })],
        size_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("array_push".to_string(), create_external_function_named(
        "array_push",
        "aether_collections_array_push",
        vec![
            ("array", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: true,
                source_location: SourceLocation::unknown(),
            }),
            ("element", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: false,
                source_location: SourceLocation::unknown(),
            }),
        ],
        bool_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("array_pop".to_string(), create_external_function_named(
        "array_pop",
        "aether_collections_array_pop",
        vec![
            ("array", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: true,
                source_location: SourceLocation::unknown(),
            }),
            ("result", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: true,
                source_location: SourceLocation::unknown(),
            }),
        ],
        bool_type.clone(),
        CallingConvention::C,
    ));
    
    // Map operations
    external_functions.insert("map_create".to_string(), create_external_function_named(
        "map_create",
        "aether_collections_map_create",
        vec![
            ("key_size", size_type.clone()),
            ("value_size", size_type.clone()),
        ],
        TypeSpecifier::Pointer {
            target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
            is_mutable: true,
            source_location: SourceLocation::unknown(),
        },
        CallingConvention::C,
    ));
    
    external_functions.insert("map_destroy".to_string(), create_external_function_named(
        "map_destroy",
        "aether_collections_map_destroy",
        vec![("map", TypeSpecifier::Pointer {
            target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
            is_mutable: true,
            source_location: SourceLocation::unknown(),
        })],
        TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            },
        CallingConvention::C,
    ));
    
    external_functions.insert("map_insert".to_string(), create_external_function_named(
        "map_insert",
        "aether_collections_map_insert",
        vec![
            ("map", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: true,
                source_location: SourceLocation::unknown(),
            }),
            ("key", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: false,
                source_location: SourceLocation::unknown(),
            }),
            ("value", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: false,
                source_location: SourceLocation::unknown(),
            }),
        ],
        bool_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("map_get".to_string(), create_external_function_named(
        "map_get",
        "aether_collections_map_get",
        vec![
            ("map", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: false,
                source_location: SourceLocation::unknown(),
            }),
            ("key", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: false,
                source_location: SourceLocation::unknown(),
            }),
            ("result", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: true,
                source_location: SourceLocation::unknown(),
            }),
        ],
        bool_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("map_contains".to_string(), create_external_function_named(
        "map_contains",
        "aether_collections_map_contains",
        vec![
            ("map", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: false,
                source_location: SourceLocation::unknown(),
            }),
            ("key", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: false,
                source_location: SourceLocation::unknown(),
            }),
        ],
        bool_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("map_remove".to_string(), create_external_function_named(
        "map_remove",
        "aether_collections_map_remove",
        vec![
            ("map", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: true,
                source_location: SourceLocation::unknown(),
            }),
            ("key", TypeSpecifier::Pointer {
                target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
                is_mutable: false,
                source_location: SourceLocation::unknown(),
            }),
        ],
        bool_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("map_size".to_string(), create_external_function_named(
        "map_size",
        "aether_collections_map_size",
        vec![("map", TypeSpecifier::Pointer {
            target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            }),
            is_mutable: false,
            source_location: SourceLocation::unknown(),
        })],
        size_type.clone(),
        CallingConvention::C,
    ));
    
    // High-level utility functions (implemented in AetherScript)
    functions.insert("array_sort".to_string(), create_function_stub(
        "array_sort",
        vec![("array", generic_array_type.clone())],
        TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            },
    ));
    
    functions.insert("array_reverse".to_string(), create_function_stub(
        "array_reverse",
        vec![("array", generic_array_type.clone())],
        TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            },
    ));
    
    functions.insert("array_find".to_string(), create_function_stub(
        "array_find",
        vec![
            ("array", generic_array_type.clone()),
            ("element", generic_element_type.clone()),
        ],
        int_type.clone(), // Returns index or -1 if not found
    ));
    
    functions.insert("array_filter".to_string(), create_function_stub(
        "array_filter",
        vec![
            ("array", generic_array_type.clone()),
            ("predicate", TypeSpecifier::Function {
                parameter_types: vec![Box::new(generic_element_type.clone())],
                return_type: Box::new(bool_type.clone()),
                source_location: SourceLocation::unknown(),
            }),
        ],
        generic_array_type.clone(),
    ));
    
    functions.insert("array_map".to_string(), create_function_stub(
        "array_map",
        vec![
            ("array", generic_array_type.clone()),
            ("transform", TypeSpecifier::Function {
                parameter_types: vec![Box::new(generic_element_type.clone())],
                return_type: Box::new(TypeSpecifier::TypeParameter {
                    name: Identifier::new("U".to_string(), SourceLocation::unknown()),
                    constraints: vec![],
                    source_location: SourceLocation::unknown(),
                }),
                source_location: SourceLocation::unknown(),
            }),
        ],
        TypeSpecifier::Array {
            element_type: Box::new(TypeSpecifier::TypeParameter {
                name: Identifier::new("U".to_string(), SourceLocation::unknown()),
                constraints: vec![],
                source_location: SourceLocation::unknown(),
            }),
            size: None,
            source_location: SourceLocation::unknown(),
        },
    ));
    
    functions.insert("array_reduce".to_string(), create_function_stub(
        "array_reduce",
        vec![
            ("array", generic_array_type.clone()),
            ("initial", TypeSpecifier::TypeParameter {
                name: Identifier::new("U".to_string(), SourceLocation::unknown()),
                constraints: vec![],
                source_location: SourceLocation::unknown(),
            }),
            ("reducer", TypeSpecifier::Function {
                parameter_types: vec![
                    Box::new(TypeSpecifier::TypeParameter {
                        name: Identifier::new("U".to_string(), SourceLocation::unknown()),
                        constraints: vec![],
                        source_location: SourceLocation::unknown(),
                    }),
                    Box::new(generic_element_type.clone()),
                ],
                return_type: Box::new(TypeSpecifier::TypeParameter {
                    name: Identifier::new("U".to_string(), SourceLocation::unknown()),
                    constraints: vec![],
                    source_location: SourceLocation::unknown(),
                }),
                source_location: SourceLocation::unknown(),
            }),
        ],
        TypeSpecifier::TypeParameter {
            name: Identifier::new("U".to_string(), SourceLocation::unknown()),
            constraints: vec![],
            source_location: SourceLocation::unknown(),
        },
    ));
    
    functions.insert("map_keys".to_string(), create_function_stub(
        "map_keys",
        vec![("map", generic_map_type.clone())],
        TypeSpecifier::Array {
            element_type: Box::new(TypeSpecifier::TypeParameter {
                name: Identifier::new("K".to_string(), SourceLocation::unknown()),
                constraints: vec![],
                source_location: SourceLocation::unknown(),
            }),
            size: None,
            source_location: SourceLocation::unknown(),
        },
    ));
    
    functions.insert("map_values".to_string(), create_function_stub(
        "map_values",
        vec![("map", generic_map_type.clone())],
        TypeSpecifier::Array {
            element_type: Box::new(TypeSpecifier::TypeParameter {
                name: Identifier::new("V".to_string(), SourceLocation::unknown()),
                constraints: vec![],
                source_location: SourceLocation::unknown(),
            }),
            size: None,
            source_location: SourceLocation::unknown(),
        },
    ));
    
    Module {
        name: Identifier::new("std.collections".to_string(), SourceLocation::unknown()),
        intent: Some("Provides efficient array and map data structures with utility functions".to_string()),
        imports: vec![],
        exports: vec![
            ExportStatement::Function {
                name: Identifier::new("array_create".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("map_create".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("array_sort".to_string(), SourceLocation::unknown()),
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
    fn test_collections_module_creation() {
        let module = create_collections_module();
        
        assert_eq!(module.name.name, "std.collections");
        assert!(module.intent.is_some());
        
        // Check array operations
        assert!(module.external_functions.iter().any(|f| f.name.name == "array_create"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "array_push"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "array_pop"));
        assert!(module.function_definitions.iter().any(|f| f.name.name == "array_sort"));
        assert!(module.function_definitions.iter().any(|f| f.name.name == "array_map"));
        
        // Check map operations
        assert!(module.external_functions.iter().any(|f| f.name.name == "map_create"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "map_insert"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "map_get"));
        assert!(module.function_definitions.iter().any(|f| f.name.name == "map_keys"));
        
        // Check exports
        assert!(module.exports.iter().any(|e| matches!(e, ExportStatement::Function { name, .. } if name.name == "array_create")));
        assert!(module.exports.iter().any(|e| matches!(e, ExportStatement::Function { name, .. } if name.name == "map_create")));
        assert!(module.exports.iter().any(|e| matches!(e, ExportStatement::Function { name, .. } if name.name == "array_sort")));
    }
    
    #[test]
    fn test_array_function_signatures() {
        let module = create_collections_module();
        
        // Test array_create function
        let array_create = module.external_functions.iter()
            .find(|f| f.name.name == "array_create")
            .expect("array_create function not found");
        assert_eq!(array_create.name.name, "array_create");
        assert_eq!(array_create.parameters.len(), 2);
        
        // Test array_push function
        let array_push = module.external_functions.iter()
            .find(|f| f.name.name == "array_push")
            .expect("array_push function not found");
        assert_eq!(array_push.parameters.len(), 2);
        assert!(matches!(array_push.return_type.as_ref(), TypeSpecifier::Primitive { type_name: PrimitiveType::Boolean, .. }));
    }
    
    #[test]
    fn test_map_function_signatures() {
        let module = create_collections_module();
        
        // Test map_create function
        let map_create = module.external_functions.iter()
            .find(|f| f.name.name == "map_create")
            .expect("map_create function not found");
        assert_eq!(map_create.name.name, "map_create");
        assert_eq!(map_create.parameters.len(), 2);
        
        // Test map_insert function
        let map_insert = module.external_functions.iter()
            .find(|f| f.name.name == "map_insert")
            .expect("map_insert function not found");
        assert_eq!(map_insert.parameters.len(), 3);
        assert!(matches!(map_insert.return_type.as_ref(), TypeSpecifier::Primitive { type_name: PrimitiveType::Boolean, .. }));
    }
}
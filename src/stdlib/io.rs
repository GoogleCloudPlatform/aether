// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! std.io - File and I/O operations module

use super::{create_external_function_named, create_function_stub};
use crate::ast::CallingConvention;
use crate::ast::{ExportStatement, Identifier, Module, PrimitiveType, TypeSpecifier};
use crate::error::SourceLocation;
use std::collections::HashMap;

/// Create the std.io module with file operations
pub fn create_io_module() -> Module {
    let mut external_functions = HashMap::new();

    // File handle type (opaque pointer)
    let file_handle_type = TypeSpecifier::Pointer {
        target_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: SourceLocation::unknown(),
        }),
        is_mutable: true,
        source_location: SourceLocation::unknown(),
    };

    // string type for convenience
    let string_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::String,
        source_location: SourceLocation::unknown(),
    };
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

    // File operations
    external_functions.insert(
        "open_file".to_string(),
        create_external_function_named(
            "open_file",
            "aether_io_open_file",
            vec![("path", string_type.clone()), ("mode", string_type.clone())],
            file_handle_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "close_file".to_string(),
        create_external_function_named(
            "close_file",
            "aether_io_close_file",
            vec![("handle", file_handle_type.clone())],
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            },
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "read_file".to_string(),
        create_external_function_named(
            "read_file",
            "aether_io_read_file",
            vec![
                ("handle", file_handle_type.clone()),
                (
                    "buffer",
                    TypeSpecifier::Pointer {
                        target_type: Box::new(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::String,
                            source_location: SourceLocation::unknown(),
                        }),
                        is_mutable: true,
                        source_location: SourceLocation::unknown(),
                    },
                ),
                ("max_size", size_type.clone()),
            ],
            int_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "write_file".to_string(),
        create_external_function_named(
            "write_file",
            "aether_io_write_file",
            vec![
                ("handle", file_handle_type.clone()),
                ("data", string_type.clone()),
            ],
            int_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "file_exists".to_string(),
        create_external_function_named(
            "file_exists",
            "aether_io_file_exists",
            vec![("path", string_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "file_size".to_string(),
        create_external_function_named(
            "file_size",
            "aether_io_file_size",
            vec![("path", string_type.clone())],
            size_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "create_directory".to_string(),
        create_external_function_named(
            "create_directory",
            "aether_io_create_directory",
            vec![("path", string_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "remove_file".to_string(),
        create_external_function_named(
            "remove_file",
            "aether_io_remove_file",
            vec![("path", string_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    // High-level convenience functions (would be implemented in AetherScript)
    let mut functions = HashMap::new();

    functions.insert(
        "read_entire_file".to_string(),
        create_function_stub(
            "read_entire_file",
            vec![("path", string_type.clone())],
            string_type.clone(),
        ),
    );

    functions.insert(
        "write_entire_file".to_string(),
        create_function_stub(
            "write_entire_file",
            vec![
                ("path", string_type.clone()),
                ("content", string_type.clone()),
            ],
            bool_type.clone(),
        ),
    );

    functions.insert(
        "append_to_file".to_string(),
        create_function_stub(
            "append_to_file",
            vec![
                ("path", string_type.clone()),
                ("content", string_type.clone()),
            ],
            bool_type.clone(),
        ),
    );

    functions.insert(
        "copy_file".to_string(),
        create_function_stub(
            "copy_file",
            vec![
                ("source", string_type.clone()),
                ("destination", string_type.clone()),
            ],
            bool_type.clone(),
        ),
    );

    Module {
        name: Identifier::new("std.io".to_string(), SourceLocation::unknown()),
        intent: Some("Provides file and I/O operations for AetherScript programs".to_string()),
        imports: vec![],
        exports: vec![
            ExportStatement::Function {
                name: Identifier::new("open_file".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("read_entire_file".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
        ],
        type_definitions: vec![],
        trait_definitions: vec![],
        impl_blocks: vec![],
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
    fn test_io_module_creation() {
        let module = create_io_module();

        assert_eq!(module.name.name, "std.io");
        assert!(module.intent.is_some());

        // Check that external functions are defined
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "open_file"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "close_file"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "read_file"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "write_file"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "file_exists"));

        // Check that high-level functions are defined
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "read_entire_file"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "write_entire_file"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "copy_file"));

        // Check exports
        assert!(module.exports.iter().any(
            |e| matches!(e, ExportStatement::Function { name, .. } if name.name == "open_file")
        ));
        assert!(module.exports.iter().any(|e| matches!(e, ExportStatement::Function { name, .. } if name.name == "read_entire_file")));
    }

    #[test]
    fn test_external_function_signatures() {
        let module = create_io_module();

        // Test open_file function signature
        let open_file = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "open_file")
            .expect("open_file function not found");
        assert_eq!(open_file.name.name, "open_file");
        assert_eq!(open_file.parameters.len(), 2);
        assert_eq!(open_file.parameters[0].name.name, "path");
        assert_eq!(open_file.parameters[1].name.name, "mode");

        // Test file_exists function signature
        let file_exists = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "file_exists")
            .expect("file_exists function not found");
        assert_eq!(file_exists.parameters.len(), 1);
        assert_eq!(file_exists.parameters[0].name.name, "path");
        assert!(matches!(
            file_exists.return_type.as_ref(),
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Boolean,
                ..
            }
        ));
    }
}

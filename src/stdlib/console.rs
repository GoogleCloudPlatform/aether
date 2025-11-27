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

//! std.console - Console input/output operations

use super::{create_external_function_named, create_function_stub};
use crate::ast::CallingConvention;
use crate::ast::{ExportStatement, Identifier, Module, PrimitiveType, TypeSpecifier};
use crate::error::SourceLocation;
use std::collections::HashMap;

/// Create the std.console module with console operations
pub fn create_console_module() -> Module {
    let mut external_functions = HashMap::new();
    let mut functions = HashMap::new();

    // Common types
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
    let void_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Void,
        source_location: SourceLocation::unknown(),
    };
    let float_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Float,
        source_location: SourceLocation::unknown(),
    };

    // Basic console output (external C functions for immediate output)
    external_functions.insert(
        "print_string".to_string(),
        create_external_function_named(
            "print_string",
            "aether_console_print_string",
            vec![("text", string_type.clone())],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "print_int".to_string(),
        create_external_function_named(
            "print_int",
            "aether_console_print_int",
            vec![("value", int_type.clone())],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "print_float".to_string(),
        create_external_function_named(
            "print_float",
            "aether_console_print_float",
            vec![("value", float_type.clone())],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "print_bool".to_string(),
        create_external_function_named(
            "print_bool",
            "aether_console_print_bool",
            vec![("value", bool_type.clone())],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "print_newline".to_string(),
        create_external_function_named(
            "print_newline",
            "aether_console_print_newline",
            vec![],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    // Console input functions
    external_functions.insert(
        "read_line".to_string(),
        create_external_function_named(
            "read_line",
            "aether_console_read_line",
            vec![],
            string_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "read_char".to_string(),
        create_external_function_named(
            "read_char",
            "aether_console_read_char",
            vec![],
            int_type.clone(), // Character as integer
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "read_int".to_string(),
        create_external_function_named(
            "read_int",
            "aether_console_read_int",
            vec![],
            int_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "read_float".to_string(),
        create_external_function_named(
            "read_float",
            "aether_console_read_float",
            vec![],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    // Console control functions
    external_functions.insert(
        "clear_screen".to_string(),
        create_external_function_named(
            "clear_screen",
            "aether_console_clear_screen",
            vec![],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "set_cursor_position".to_string(),
        create_external_function_named(
            "set_cursor_position",
            "aether_console_set_cursor_position",
            vec![("row", int_type.clone()), ("column", int_type.clone())],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "get_cursor_position".to_string(),
        create_external_function_named(
            "get_cursor_position",
            "aether_console_get_cursor_position",
            vec![
                (
                    "row",
                    TypeSpecifier::Pointer {
                        target_type: Box::new(int_type.clone()),
                        is_mutable: true,
                        source_location: SourceLocation::unknown(),
                    },
                ),
                (
                    "column",
                    TypeSpecifier::Pointer {
                        target_type: Box::new(int_type.clone()),
                        is_mutable: true,
                        source_location: SourceLocation::unknown(),
                    },
                ),
            ],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "set_text_color".to_string(),
        create_external_function_named(
            "set_text_color",
            "aether_console_set_text_color",
            vec![("color_code", int_type.clone())],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "set_background_color".to_string(),
        create_external_function_named(
            "set_background_color",
            "aether_console_set_background_color",
            vec![("color_code", int_type.clone())],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "reset_colors".to_string(),
        create_external_function_named(
            "reset_colors",
            "aether_console_reset_colors",
            vec![],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "key_available".to_string(),
        create_external_function_named(
            "key_available",
            "aether_console_key_available",
            vec![],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "get_console_size".to_string(),
        create_external_function_named(
            "get_console_size",
            "aether_console_get_size",
            vec![
                (
                    "width",
                    TypeSpecifier::Pointer {
                        target_type: Box::new(int_type.clone()),
                        is_mutable: true,
                        source_location: SourceLocation::unknown(),
                    },
                ),
                (
                    "height",
                    TypeSpecifier::Pointer {
                        target_type: Box::new(int_type.clone()),
                        is_mutable: true,
                        source_location: SourceLocation::unknown(),
                    },
                ),
            ],
            void_type.clone(),
            CallingConvention::C,
        ),
    );

    // High-level convenience functions (implemented in AetherScript)
    functions.insert(
        "println".to_string(),
        create_function_stub(
            "println",
            vec![("text", string_type.clone())],
            void_type.clone(),
        ),
    );

    functions.insert(
        "print".to_string(),
        create_function_stub(
            "print",
            vec![("text", string_type.clone())],
            void_type.clone(),
        ),
    );

    functions.insert(
        "printf".to_string(),
        create_function_stub(
            "printf",
            vec![
                ("format", string_type.clone()),
                // Would need variadic support for full printf
            ],
            void_type.clone(),
        ),
    );

    functions.insert(
        "input".to_string(),
        create_function_stub(
            "input",
            vec![("prompt", string_type.clone())],
            string_type.clone(),
        ),
    );

    functions.insert(
        "confirm".to_string(),
        create_function_stub(
            "confirm",
            vec![("prompt", string_type.clone())],
            bool_type.clone(),
        ),
    );

    functions.insert(
        "pause".to_string(),
        create_function_stub(
            "pause",
            vec![("message", string_type.clone())],
            void_type.clone(),
        ),
    );

    // Color constants and utilities
    functions.insert(
        "black".to_string(),
        create_function_stub("black", vec![], int_type.clone()),
    );

    functions.insert(
        "red".to_string(),
        create_function_stub("red", vec![], int_type.clone()),
    );

    functions.insert(
        "green".to_string(),
        create_function_stub("green", vec![], int_type.clone()),
    );

    functions.insert(
        "yellow".to_string(),
        create_function_stub("yellow", vec![], int_type.clone()),
    );

    functions.insert(
        "blue".to_string(),
        create_function_stub("blue", vec![], int_type.clone()),
    );

    functions.insert(
        "magenta".to_string(),
        create_function_stub("magenta", vec![], int_type.clone()),
    );

    functions.insert(
        "cyan".to_string(),
        create_function_stub("cyan", vec![], int_type.clone()),
    );

    functions.insert(
        "white".to_string(),
        create_function_stub("white", vec![], int_type.clone()),
    );

    // Text formatting functions
    functions.insert(
        "bold".to_string(),
        create_function_stub(
            "bold",
            vec![("text", string_type.clone())],
            string_type.clone(),
        ),
    );

    functions.insert(
        "italic".to_string(),
        create_function_stub(
            "italic",
            vec![("text", string_type.clone())],
            string_type.clone(),
        ),
    );

    functions.insert(
        "underline".to_string(),
        create_function_stub(
            "underline",
            vec![("text", string_type.clone())],
            string_type.clone(),
        ),
    );

    functions.insert(
        "strikethrough".to_string(),
        create_function_stub(
            "strikethrough",
            vec![("text", string_type.clone())],
            string_type.clone(),
        ),
    );

    // Progress and status functions
    functions.insert(
        "progress_bar".to_string(),
        create_function_stub(
            "progress_bar",
            vec![
                ("current", int_type.clone()),
                ("total", int_type.clone()),
                ("width", int_type.clone()),
            ],
            void_type.clone(),
        ),
    );

    functions.insert(
        "spinner".to_string(),
        create_function_stub(
            "spinner",
            vec![("message", string_type.clone())],
            void_type.clone(),
        ),
    );

    functions.insert(
        "error_message".to_string(),
        create_function_stub(
            "error_message",
            vec![("message", string_type.clone())],
            void_type.clone(),
        ),
    );

    functions.insert(
        "warning_message".to_string(),
        create_function_stub(
            "warning_message",
            vec![("message", string_type.clone())],
            void_type.clone(),
        ),
    );

    functions.insert(
        "success_message".to_string(),
        create_function_stub(
            "success_message",
            vec![("message", string_type.clone())],
            void_type.clone(),
        ),
    );

    functions.insert(
        "info_message".to_string(),
        create_function_stub(
            "info_message",
            vec![("message", string_type.clone())],
            void_type.clone(),
        ),
    );

    Module {
        name: Identifier::new("std.console".to_string(), SourceLocation::unknown()),
        intent: Some("Provides console input/output operations and terminal control".to_string()),
        imports: vec![],
        exports: vec![
            // Basic output
            ExportStatement::Function {
                name: Identifier::new("print_string".to_string(), SourceLocation::unknown()),
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
    fn test_console_module_creation() {
        let module = create_console_module();

        assert_eq!(module.name.name, "std.console");
        assert!(module.intent.is_some());

        // Check basic output functions
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "print_string"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "print_int"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "print_float"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "print_bool"));

        // Check input functions
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "read_line"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "read_char"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "read_int"));

        // Check high-level functions
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "println"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "input"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "confirm"));

        // Check exports
        assert!(module.exports.len() > 0);
    }

    #[test]
    fn test_console_output_functions() {
        let module = create_console_module();

        // Test print_string function
        let print_string = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "print_string")
            .unwrap();
        assert_eq!(print_string.name.name, "print_string");
        assert_eq!(print_string.parameters.len(), 1);
        assert_eq!(print_string.parameters[0].name.name, "text");

        // Test print_int function
        let print_int = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "print_int")
            .unwrap();
        assert_eq!(print_int.parameters.len(), 1);
        assert_eq!(print_int.parameters[0].name.name, "value");
        assert!(matches!(
            print_int.return_type.as_ref(),
            TypeSpecifier::Primitive {
                type_name: crate::ast::PrimitiveType::Void,
                ..
            }
        ));
    }

    #[test]
    fn test_console_input_functions() {
        let module = create_console_module();

        // Test read_line function
        let read_line = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "read_line")
            .unwrap();
        assert_eq!(read_line.parameters.len(), 0);
        assert!(matches!(
            read_line.return_type.as_ref(),
            TypeSpecifier::Primitive {
                type_name: crate::ast::PrimitiveType::String,
                ..
            }
        ));

        // Test read_int function
        let read_int = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "read_int")
            .unwrap();
        assert_eq!(read_int.parameters.len(), 0);
        assert!(matches!(
            read_int.return_type.as_ref(),
            TypeSpecifier::Primitive {
                type_name: crate::ast::PrimitiveType::Integer,
                ..
            }
        ));
    }

    #[test]
    fn test_console_control_functions() {
        let module = create_console_module();

        // Test clear_screen function
        let clear_screen = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "clear_screen")
            .unwrap();
        assert_eq!(clear_screen.parameters.len(), 0);

        // Test set_cursor_position function
        let set_cursor = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "set_cursor_position")
            .unwrap();
        assert_eq!(set_cursor.parameters.len(), 2);
        assert_eq!(set_cursor.parameters[0].name.name, "row");
        assert_eq!(set_cursor.parameters[1].name.name, "column");

        // Test color functions
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "set_text_color"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "reset_colors"));
    }

    #[test]
    fn test_high_level_functions() {
        let module = create_console_module();

        // Test input function
        let input_func = module
            .function_definitions
            .iter()
            .find(|f| f.name.name == "input")
            .unwrap();
        assert_eq!(input_func.parameters.len(), 1);
        assert_eq!(input_func.parameters[0].name.name, "prompt");
        assert!(matches!(
            input_func.return_type.as_ref(),
            TypeSpecifier::Primitive {
                type_name: crate::ast::PrimitiveType::String,
                ..
            }
        ));

        // Test confirm function
        let confirm_func = module
            .function_definitions
            .iter()
            .find(|f| f.name.name == "confirm")
            .unwrap();
        assert_eq!(confirm_func.parameters.len(), 1);
        assert!(matches!(
            confirm_func.return_type.as_ref(),
            TypeSpecifier::Primitive {
                type_name: crate::ast::PrimitiveType::Boolean,
                ..
            }
        ));

        // Test color functions
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "red"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "blue"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "green"));
    }
}

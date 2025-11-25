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

//! std.json - JSON parsing and serialization module

use super::{create_external_function_named, create_function_stub};
use crate::ast::CallingConvention;
use crate::ast::{ExportStatement, Identifier, Module, PrimitiveType, TypeSpecifier};
use crate::error::SourceLocation;
use std::collections::HashMap;

/// Create the std.json module with JSON operations
pub fn create_json_module() -> Module {
    let mut external_functions = HashMap::new();
    let mut functions = HashMap::new();

    // Common types
    let string_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::String,
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
    let float_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Float,
        source_location: SourceLocation::unknown(),
    };

    // JSON value type (placeholder - should be a variant type)
    let json_value_type = string_type.clone(); // TODO: Should be a proper JSON value type

    // JSON parsing functions (external for performance)
    external_functions.insert(
        "parse_json".to_string(),
        create_external_function_named(
            "parse_json",
            "aether_json_parse",
            vec![("json_string", string_type.clone())],
            json_value_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "stringify_json".to_string(),
        create_external_function_named(
            "stringify_json",
            "aether_json_stringify",
            vec![("value", json_value_type.clone())],
            string_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "validate_json".to_string(),
        create_external_function_named(
            "validate_json",
            "aether_json_validate",
            vec![("json_string", string_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    // JSON manipulation functions
    external_functions.insert(
        "json_get_field".to_string(),
        create_external_function_named(
            "json_get_field",
            "aether_json_get_field",
            vec![
                ("json_value", json_value_type.clone()),
                ("field_name", string_type.clone()),
            ],
            json_value_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "json_set_field".to_string(),
        create_external_function_named(
            "json_set_field",
            "aether_json_set_field",
            vec![
                ("json_value", json_value_type.clone()),
                ("field_name", string_type.clone()),
                ("new_value", json_value_type.clone()),
            ],
            json_value_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "json_array_get".to_string(),
        create_external_function_named(
            "json_array_get",
            "aether_json_array_get",
            vec![
                ("json_array", json_value_type.clone()),
                ("index", int_type.clone()),
            ],
            json_value_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "json_array_push".to_string(),
        create_external_function_named(
            "json_array_push",
            "aether_json_array_push",
            vec![
                ("json_array", json_value_type.clone()),
                ("value", json_value_type.clone()),
            ],
            json_value_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "json_array_length".to_string(),
        create_external_function_named(
            "json_array_length",
            "aether_json_array_length",
            vec![("json_array", json_value_type.clone())],
            int_type.clone(),
            CallingConvention::C,
        ),
    );

    // Type checking functions
    external_functions.insert(
        "json_is_null".to_string(),
        create_external_function_named(
            "json_is_null",
            "aether_json_is_null",
            vec![("value", json_value_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "json_is_boolean".to_string(),
        create_external_function_named(
            "json_is_boolean",
            "aether_json_is_boolean",
            vec![("value", json_value_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "json_is_number".to_string(),
        create_external_function_named(
            "json_is_number",
            "aether_json_is_number",
            vec![("value", json_value_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "json_is_string".to_string(),
        create_external_function_named(
            "json_is_string",
            "aether_json_is_string",
            vec![("value", json_value_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "json_is_array".to_string(),
        create_external_function_named(
            "json_is_array",
            "aether_json_is_array",
            vec![("value", json_value_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "json_is_object".to_string(),
        create_external_function_named(
            "json_is_object",
            "aether_json_is_object",
            vec![("value", json_value_type.clone())],
            bool_type.clone(),
            CallingConvention::C,
        ),
    );

    // High-level JSON functions (implemented in AetherScript)
    functions.insert(
        "create_object".to_string(),
        create_function_stub("create_object", vec![], json_value_type.clone()),
    );

    functions.insert(
        "create_array".to_string(),
        create_function_stub("create_array", vec![], json_value_type.clone()),
    );

    functions.insert(
        "from_string".to_string(),
        create_function_stub(
            "from_string",
            vec![("value", string_type.clone())],
            json_value_type.clone(),
        ),
    );

    functions.insert(
        "from_integer".to_string(),
        create_function_stub(
            "from_integer",
            vec![("value", int_type.clone())],
            json_value_type.clone(),
        ),
    );

    functions.insert(
        "from_float".to_string(),
        create_function_stub(
            "from_float",
            vec![("value", float_type.clone())],
            json_value_type.clone(),
        ),
    );

    functions.insert(
        "from_boolean".to_string(),
        create_function_stub(
            "from_boolean",
            vec![("value", bool_type.clone())],
            json_value_type.clone(),
        ),
    );

    functions.insert(
        "to_string".to_string(),
        create_function_stub(
            "to_string",
            vec![("json_value", json_value_type.clone())],
            string_type.clone(),
        ),
    );

    functions.insert(
        "to_integer".to_string(),
        create_function_stub(
            "to_integer",
            vec![("json_value", json_value_type.clone())],
            int_type.clone(),
        ),
    );

    functions.insert(
        "to_float".to_string(),
        create_function_stub(
            "to_float",
            vec![("json_value", json_value_type.clone())],
            float_type.clone(),
        ),
    );

    functions.insert(
        "to_boolean".to_string(),
        create_function_stub(
            "to_boolean",
            vec![("json_value", json_value_type.clone())],
            bool_type.clone(),
        ),
    );

    // JSON path queries
    functions.insert(
        "query".to_string(),
        create_function_stub(
            "query",
            vec![
                ("json_value", json_value_type.clone()),
                ("path", string_type.clone()),
            ],
            json_value_type.clone(),
        ),
    );

    functions.insert(
        "exists".to_string(),
        create_function_stub(
            "exists",
            vec![
                ("json_value", json_value_type.clone()),
                ("path", string_type.clone()),
            ],
            bool_type.clone(),
        ),
    );

    // JSON merging and manipulation
    functions.insert(
        "merge".to_string(),
        create_function_stub(
            "merge",
            vec![
                ("target", json_value_type.clone()),
                ("source", json_value_type.clone()),
            ],
            json_value_type.clone(),
        ),
    );

    functions.insert(
        "deep_merge".to_string(),
        create_function_stub(
            "deep_merge",
            vec![
                ("target", json_value_type.clone()),
                ("source", json_value_type.clone()),
            ],
            json_value_type.clone(),
        ),
    );

    functions.insert(
        "clone".to_string(),
        create_function_stub(
            "clone",
            vec![("json_value", json_value_type.clone())],
            json_value_type.clone(),
        ),
    );

    functions.insert(
        "pretty_print".to_string(),
        create_function_stub(
            "pretty_print",
            vec![
                ("json_value", json_value_type.clone()),
                ("indent", int_type.clone()),
            ],
            string_type.clone(),
        ),
    );

    Module {
        name: Identifier::new("std.json".to_string(), SourceLocation::unknown()),
        intent: Some(
            "Provides JSON parsing, serialization, and manipulation utilities".to_string(),
        ),
        imports: vec![],
        exports: vec![
            ExportStatement::Function {
                name: Identifier::new("parse_json".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("stringify_json".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("create_object".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("query".to_string(), SourceLocation::unknown()),
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
    fn test_json_module_creation() {
        let module = create_json_module();

        assert_eq!(module.name.name, "std.json");
        assert!(module.intent.is_some());

        // Check parsing functions
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "parse_json"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "stringify_json"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "validate_json"));

        // Check manipulation functions
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "json_get_field"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "json_array_push"));

        // Check high-level functions
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "create_object"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "query"));

        // Check exports
        assert!(module.exports.iter().any(
            |e| matches!(e, ExportStatement::Function { name, .. } if name.name == "parse_json")
        ));
    }

    #[test]
    fn test_json_function_signatures() {
        let module = create_json_module();

        // Test parse_json function
        let parse_json = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "parse_json")
            .expect("parse_json function not found");
        assert_eq!(parse_json.parameters.len(), 1);
        assert_eq!(parse_json.parameters[0].name.name, "json_string");

        // Test json_get_field function
        let get_field = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "json_get_field")
            .expect("json_get_field function not found");
        assert_eq!(get_field.parameters.len(), 2);
        assert_eq!(get_field.parameters[0].name.name, "json_value");
        assert_eq!(get_field.parameters[1].name.name, "field_name");
    }
}

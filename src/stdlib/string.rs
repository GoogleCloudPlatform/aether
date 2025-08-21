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

//! std.string - String manipulation and utility functions

use crate::ast::{Module, TypeSpecifier, PrimitiveType, Identifier, ExportStatement};
use crate::error::SourceLocation;
use crate::ast::CallingConvention;
use super::{create_external_function_named, create_function_stub};
use std::collections::HashMap;

/// Create the std.string module with string operations
pub fn create_string_module() -> Module {
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
    let size_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::SizeT,
        source_location: SourceLocation::unknown(),
    };
    let char_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Integer,
        source_location: SourceLocation::unknown(),
    }; // Character as integer
    
    // String array type
    let string_array_type = TypeSpecifier::Array {
        element_type: Box::new(string_type.clone()),
        size: None,
        source_location: SourceLocation::unknown(),
    };
    
    // Basic string operations (external for performance)
    external_functions.insert("string_length".to_string(), create_external_function_named(
        "string_length",
        "aether_string_length",
        vec![("str", string_type.clone())],
        size_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_concat".to_string(), create_external_function_named(
        "string_concat",
        "aether_string_concat",
        vec![
            ("str1", string_type.clone()),
            ("str2", string_type.clone()),
        ],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_compare".to_string(), create_external_function_named(
        "string_compare",
        "aether_string_compare",
        vec![
            ("str1", string_type.clone()),
            ("str2", string_type.clone()),
        ],
        int_type.clone(), // Returns -1, 0, or 1
        CallingConvention::C,
    ));
    
    external_functions.insert("string_copy".to_string(), create_external_function_named(
        "string_copy",
        "aether_string_copy",
        vec![("str", string_type.clone())],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_substring".to_string(), create_external_function_named(
        "string_substring",
        "aether_string_substring",
        vec![
            ("str", string_type.clone()),
            ("start", size_type.clone()),
            ("length", size_type.clone()),
        ],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_find".to_string(), create_external_function_named(
        "string_find",
        "aether_string_find",
        vec![
            ("haystack", string_type.clone()),
            ("needle", string_type.clone()),
        ],
        int_type.clone(), // Returns index or -1 if not found
        CallingConvention::C,
    ));
    
    external_functions.insert("string_find_from".to_string(), create_external_function_named(
        "string_find_from",
        "aether_string_find_from",
        vec![
            ("haystack", string_type.clone()),
            ("needle", string_type.clone()),
            ("start_index", size_type.clone()),
        ],
        int_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_replace".to_string(), create_external_function_named(
        "string_replace",
        "aether_string_replace",
        vec![
            ("str", string_type.clone()),
            ("old_substr", string_type.clone()),
            ("new_substr", string_type.clone()),
        ],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_to_lower".to_string(), create_external_function_named(
        "string_to_lower",
        "aether_string_to_lower",
        vec![("str", string_type.clone())],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_to_upper".to_string(), create_external_function_named(
        "string_to_upper",
        "aether_string_to_upper",
        vec![("str", string_type.clone())],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_trim".to_string(), create_external_function_named(
        "string_trim",
        "aether_string_trim",
        vec![("str", string_type.clone())],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_trim_left".to_string(), create_external_function_named(
        "string_trim_left",
        "aether_string_trim_left",
        vec![("str", string_type.clone())],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_trim_right".to_string(), create_external_function_named(
        "string_trim_right",
        "aether_string_trim_right",
        vec![("str", string_type.clone())],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_split".to_string(), create_external_function_named(
        "string_split",
        "aether_string_split",
        vec![
            ("str", string_type.clone()),
            ("delimiter", string_type.clone()),
            ("max_splits", int_type.clone()),
        ],
        string_array_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_join".to_string(), create_external_function_named(
        "string_join",
        "aether_string_join",
        vec![
            ("strings", string_array_type.clone()),
            ("separator", string_type.clone()),
        ],
        string_type.clone(),
        CallingConvention::C,
    ));
    
    external_functions.insert("string_char_at".to_string(), create_external_function_named(
        "string_char_at",
        "aether_string_char_at",
        vec![
            ("str", string_type.clone()),
            ("index", size_type.clone()),
        ],
        char_type.clone(),
        CallingConvention::C,
    ));
    
    // High-level utility functions (implemented in AetherScript)
    functions.insert("starts_with".to_string(), create_function_stub(
        "starts_with",
        vec![
            ("str", string_type.clone()),
            ("prefix", string_type.clone()),
        ],
        bool_type.clone(),
    ));
    
    functions.insert("ends_with".to_string(), create_function_stub(
        "ends_with",
        vec![
            ("str", string_type.clone()),
            ("suffix", string_type.clone()),
        ],
        bool_type.clone(),
    ));
    
    functions.insert("contains".to_string(), create_function_stub(
        "contains",
        vec![
            ("str", string_type.clone()),
            ("substr", string_type.clone()),
        ],
        bool_type.clone(),
    ));
    
    functions.insert("is_empty".to_string(), create_function_stub(
        "is_empty",
        vec![("str", string_type.clone())],
        bool_type.clone(),
    ));
    
    functions.insert("is_whitespace".to_string(), create_function_stub(
        "is_whitespace",
        vec![("str", string_type.clone())],
        bool_type.clone(),
    ));
    
    functions.insert("repeat".to_string(), create_function_stub(
        "repeat",
        vec![
            ("str", string_type.clone()),
            ("count", int_type.clone()),
        ],
        string_type.clone(),
    ));
    
    functions.insert("reverse".to_string(), create_function_stub(
        "reverse",
        vec![("str", string_type.clone())],
        string_type.clone(),
    ));
    
    functions.insert("pad_left".to_string(), create_function_stub(
        "pad_left",
        vec![
            ("str", string_type.clone()),
            ("width", int_type.clone()),
            ("pad_char", char_type.clone()),
        ],
        string_type.clone(),
    ));
    
    functions.insert("pad_right".to_string(), create_function_stub(
        "pad_right",
        vec![
            ("str", string_type.clone()),
            ("width", int_type.clone()),
            ("pad_char", char_type.clone()),
        ],
        string_type.clone(),
    ));
    
    functions.insert("center".to_string(), create_function_stub(
        "center",
        vec![
            ("str", string_type.clone()),
            ("width", int_type.clone()),
            ("pad_char", char_type.clone()),
        ],
        string_type.clone(),
    ));
    
    // String validation functions
    functions.insert("is_numeric".to_string(), create_function_stub(
        "is_numeric",
        vec![("str", string_type.clone())],
        bool_type.clone(),
    ));
    
    functions.insert("is_alphabetic".to_string(), create_function_stub(
        "is_alphabetic",
        vec![("str", string_type.clone())],
        bool_type.clone(),
    ));
    
    functions.insert("is_alphanumeric".to_string(), create_function_stub(
        "is_alphanumeric",
        vec![("str", string_type.clone())],
        bool_type.clone(),
    ));
    
    functions.insert("is_ascii".to_string(), create_function_stub(
        "is_ascii",
        vec![("str", string_type.clone())],
        bool_type.clone(),
    ));
    
    // String conversion functions
    functions.insert("to_int".to_string(), create_function_stub(
        "to_int",
        vec![("str", string_type.clone())],
        int_type.clone(),
    ));
    
    functions.insert("to_float".to_string(), create_function_stub(
        "to_float",
        vec![("str", string_type.clone())],
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Float,
            source_location: SourceLocation::unknown(),
        },
    ));
    
    functions.insert("from_int".to_string(), create_function_stub(
        "from_int",
        vec![("value", int_type.clone())],
        string_type.clone(),
    ));
    
    functions.insert("from_float".to_string(), create_function_stub(
        "from_float",
        vec![("value", TypeSpecifier::Primitive {
            type_name: PrimitiveType::Float,
            source_location: SourceLocation::unknown(),
        })],
        string_type.clone(),
    ));
    
    // Regular expression functions (simplified)
    functions.insert("match_pattern".to_string(), create_function_stub(
        "match_pattern",
        vec![
            ("str", string_type.clone()),
            ("pattern", string_type.clone()),
        ],
        bool_type.clone(),
    ));
    
    functions.insert("extract_matches".to_string(), create_function_stub(
        "extract_matches",
        vec![
            ("str", string_type.clone()),
            ("pattern", string_type.clone()),
        ],
        string_array_type.clone(),
    ));
    
    functions.insert("replace_pattern".to_string(), create_function_stub(
        "replace_pattern",
        vec![
            ("str", string_type.clone()),
            ("pattern", string_type.clone()),
            ("replacement", string_type.clone()),
        ],
        string_type.clone(),
    ));
    
    Module {
        name: Identifier::new("std.string".to_string(), SourceLocation::unknown()),
        intent: Some("Provides comprehensive string manipulation and utility functions".to_string()),
        imports: vec![],
        exports: vec![
            ExportStatement::Function {
                name: Identifier::new("string_length".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("starts_with".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("to_int".to_string(), SourceLocation::unknown()),
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
    fn test_string_module_creation() {
        let module = create_string_module();
        
        assert_eq!(module.name.name, "std.string");
        assert!(module.intent.is_some());
        
        // Check basic string operations
        assert!(module.external_functions.iter().any(|f| f.name.name == "string_length"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "string_concat"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "string_compare"));
        assert!(module.external_functions.iter().any(|f| f.name.name == "string_substring"));
        
        // Check utility functions
        assert!(module.function_definitions.iter().any(|f| f.name.name == "starts_with"));
        assert!(module.function_definitions.iter().any(|f| f.name.name == "ends_with"));
        assert!(module.function_definitions.iter().any(|f| f.name.name == "contains"));
        assert!(module.function_definitions.iter().any(|f| f.name.name == "is_numeric"));
        
        // Check exports
        assert!(module.exports.iter().any(|e| matches!(e, ExportStatement::Function { name, .. } if name.name == "string_length")));
        // Check that exported functions are present
        let export_names: Vec<String> = module.exports.iter().map(|e| match e {
            ExportStatement::Function { name, .. } => name.name.clone(),
            _ => String::new(),
        }).collect();
        assert!(export_names.contains(&"starts_with".to_string()));
        assert!(export_names.contains(&"to_int".to_string()));
    }
    
    #[test]
    fn test_string_function_signatures() {
        let module = create_string_module();
        
        // Test string_concat function
        let concat_func = module.external_functions.iter()
            .find(|f| f.name.name == "string_concat")
            .expect("string_concat function not found");
        assert_eq!(concat_func.name.name, "string_concat");
        assert_eq!(concat_func.parameters.len(), 2);
        assert_eq!(concat_func.parameters[0].name.name, "str1");
        assert_eq!(concat_func.parameters[1].name.name, "str2");
        
        // Test string_substring function
        let substring_func = module.external_functions.iter()
            .find(|f| f.name.name == "string_substring")
            .expect("string_substring function not found");
        assert_eq!(substring_func.parameters.len(), 3);
        assert_eq!(substring_func.parameters[1].name.name, "start");
        assert_eq!(substring_func.parameters[2].name.name, "length");
    }
    
    #[test]
    fn test_string_validation_functions() {
        let module = create_string_module();
        
        // Test is_numeric function
        let is_numeric = module.function_definitions.iter()
            .find(|f| f.name.name == "is_numeric")
            .expect("is_numeric function not found");
        assert_eq!(is_numeric.parameters.len(), 1);
        assert!(matches!(*is_numeric.return_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Boolean, .. }));
        
        // Test conversion functions
        let to_int = module.function_definitions.iter()
            .find(|f| f.name.name == "to_int")
            .expect("to_int function not found");
        assert!(matches!(*to_int.return_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
        
        let to_float = module.function_definitions.iter()
            .find(|f| f.name.name == "to_float")
            .expect("to_float function not found");
        assert!(matches!(*to_float.return_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Float, .. }));
    }
    
    #[test]
    fn test_string_manipulation_functions() {
        let module = create_string_module();
        
        // Test pad_left function
        let pad_left = module.function_definitions.iter()
            .find(|f| f.name.name == "pad_left")
            .expect("pad_left function not found");
        assert_eq!(pad_left.parameters.len(), 3);
        assert_eq!(pad_left.parameters[0].name.name, "str");
        assert_eq!(pad_left.parameters[1].name.name, "width");
        assert_eq!(pad_left.parameters[2].name.name, "pad_char");
        
        // Test repeat function
        let repeat = module.function_definitions.iter()
            .find(|f| f.name.name == "repeat")
            .expect("repeat function not found");
        assert_eq!(repeat.parameters.len(), 2);
        assert!(matches!(*repeat.return_type, TypeSpecifier::Primitive { type_name: PrimitiveType::String, .. }));
    }
}
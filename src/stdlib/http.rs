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

//! std.http - HTTP protocol parsing and response generation module

use super::{create_external_function_named, create_function_stub};
use crate::ast::CallingConvention;
use crate::ast::{ExportStatement, Identifier, Module, PrimitiveType, TypeSpecifier};
use crate::error::SourceLocation;
use std::collections::HashMap;

/// Create the std.http module with HTTP protocol operations
pub fn create_http_module() -> Module {
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

    // HTTP request structure type (placeholder for now)
    let request_type = string_type.clone(); // TODO: Should be a structured type
    let response_type = string_type.clone(); // TODO: Should be a structured type

    // HTTP parsing functions (external for performance)
    external_functions.insert(
        "parse_request_line".to_string(),
        create_external_function_named(
            "parse_request_line",
            "aether_http_parse_request_line",
            vec![("line", string_type.clone())],
            request_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "parse_headers".to_string(),
        create_external_function_named(
            "parse_headers",
            "aether_http_parse_headers",
            vec![("headers_text", string_type.clone())],
            string_type.clone(), // TODO: Should return header map
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "parse_url".to_string(),
        create_external_function_named(
            "parse_url",
            "aether_http_parse_url",
            vec![("url", string_type.clone())],
            string_type.clone(), // TODO: Should return URL components
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "parse_query_string".to_string(),
        create_external_function_named(
            "parse_query_string",
            "aether_http_parse_query_string",
            vec![("query", string_type.clone())],
            string_type.clone(), // TODO: Should return parameter map
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "url_encode".to_string(),
        create_external_function_named(
            "url_encode",
            "aether_http_url_encode",
            vec![("text", string_type.clone())],
            string_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "url_decode".to_string(),
        create_external_function_named(
            "url_decode",
            "aether_http_url_decode",
            vec![("encoded", string_type.clone())],
            string_type.clone(),
            CallingConvention::C,
        ),
    );

    // HTTP response building
    external_functions.insert(
        "format_response".to_string(),
        create_external_function_named(
            "format_response",
            "aether_http_format_response",
            vec![
                ("status_code", int_type.clone()),
                ("headers", string_type.clone()),
                ("body", string_type.clone()),
            ],
            string_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "format_status_line".to_string(),
        create_external_function_named(
            "format_status_line",
            "aether_http_format_status_line",
            vec![
                ("version", string_type.clone()),
                ("status_code", int_type.clone()),
                ("reason_phrase", string_type.clone()),
            ],
            string_type.clone(),
            CallingConvention::C,
        ),
    );

    // High-level HTTP functions (implemented in AetherScript)
    functions.insert(
        "parse_request".to_string(),
        create_function_stub(
            "parse_request",
            vec![("raw_request", string_type.clone())],
            request_type.clone(),
        ),
    );

    functions.insert(
        "create_response".to_string(),
        create_function_stub(
            "create_response",
            vec![
                ("status_code", int_type.clone()),
                ("body", string_type.clone()),
            ],
            response_type.clone(),
        ),
    );

    functions.insert(
        "add_header".to_string(),
        create_function_stub(
            "add_header",
            vec![
                ("response", response_type.clone()),
                ("name", string_type.clone()),
                ("value", string_type.clone()),
            ],
            response_type.clone(),
        ),
    );

    functions.insert(
        "set_content_type".to_string(),
        create_function_stub(
            "set_content_type",
            vec![
                ("response", response_type.clone()),
                ("content_type", string_type.clone()),
            ],
            response_type.clone(),
        ),
    );

    functions.insert(
        "set_content_length".to_string(),
        create_function_stub(
            "set_content_length",
            vec![
                ("response", response_type.clone()),
                ("length", size_type.clone()),
            ],
            response_type.clone(),
        ),
    );

    // Cookie handling
    functions.insert(
        "parse_cookies".to_string(),
        create_function_stub(
            "parse_cookies",
            vec![("cookie_header", string_type.clone())],
            string_type.clone(), // TODO: Should return cookie map
        ),
    );

    functions.insert(
        "set_cookie".to_string(),
        create_function_stub(
            "set_cookie",
            vec![
                ("response", response_type.clone()),
                ("name", string_type.clone()),
                ("value", string_type.clone()),
                ("options", string_type.clone()),
            ],
            response_type.clone(),
        ),
    );

    // HTTP method helpers
    functions.insert(
        "is_get".to_string(),
        create_function_stub(
            "is_get",
            vec![("request", request_type.clone())],
            bool_type.clone(),
        ),
    );

    functions.insert(
        "is_post".to_string(),
        create_function_stub(
            "is_post",
            vec![("request", request_type.clone())],
            bool_type.clone(),
        ),
    );

    functions.insert(
        "is_put".to_string(),
        create_function_stub(
            "is_put",
            vec![("request", request_type.clone())],
            bool_type.clone(),
        ),
    );

    functions.insert(
        "is_delete".to_string(),
        create_function_stub(
            "is_delete",
            vec![("request", request_type.clone())],
            bool_type.clone(),
        ),
    );

    // Content negotiation
    functions.insert(
        "accepts".to_string(),
        create_function_stub(
            "accepts",
            vec![
                ("request", request_type.clone()),
                ("content_type", string_type.clone()),
            ],
            bool_type.clone(),
        ),
    );

    functions.insert(
        "prefers".to_string(),
        create_function_stub(
            "prefers",
            vec![
                ("request", request_type.clone()),
                ("types", string_type.clone()),
            ],
            string_type.clone(),
        ),
    );

    // Status code helpers
    functions.insert(
        "redirect".to_string(),
        create_function_stub(
            "redirect",
            vec![
                ("location", string_type.clone()),
                ("permanent", bool_type.clone()),
            ],
            response_type.clone(),
        ),
    );

    functions.insert(
        "not_found".to_string(),
        create_function_stub(
            "not_found",
            vec![("message", string_type.clone())],
            response_type.clone(),
        ),
    );

    functions.insert(
        "server_error".to_string(),
        create_function_stub(
            "server_error",
            vec![("message", string_type.clone())],
            response_type.clone(),
        ),
    );

    functions.insert(
        "json_response".to_string(),
        create_function_stub(
            "json_response",
            vec![
                ("data", string_type.clone()),
                ("status_code", int_type.clone()),
            ],
            response_type.clone(),
        ),
    );

    Module {
        name: Identifier::new("std.http".to_string(), SourceLocation::unknown()),
        intent: Some(
            "Provides HTTP protocol parsing and response generation utilities".to_string(),
        ),
        imports: vec![],
        exports: vec![
            ExportStatement::Function {
                name: Identifier::new("parse_request".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("create_response".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("json_response".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("redirect".to_string(), SourceLocation::unknown()),
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
    fn test_http_module_creation() {
        let module = create_http_module();

        assert_eq!(module.name.name, "std.http");
        assert!(module.intent.is_some());

        // Check parsing functions
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "parse_request_line"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "parse_headers"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "url_encode"));

        // Check high-level functions
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "parse_request"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "create_response"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "json_response"));

        // Check exports
        assert!(module.exports.iter().any(
            |e| matches!(e, ExportStatement::Function { name, .. } if name.name == "parse_request")
        ));
    }

    #[test]
    fn test_http_function_signatures() {
        let module = create_http_module();

        // Test parse_request_line function
        let parse_line = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "parse_request_line")
            .expect("parse_request_line function not found");
        assert_eq!(parse_line.parameters.len(), 1);
        assert_eq!(parse_line.parameters[0].name.name, "line");

        // Test create_response function
        let create_resp = module
            .function_definitions
            .iter()
            .find(|f| f.name.name == "create_response")
            .expect("create_response function not found");
        assert_eq!(create_resp.parameters.len(), 2);
        assert_eq!(create_resp.parameters[0].name.name, "status_code");
        assert_eq!(create_resp.parameters[1].name.name, "body");
    }
}

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

//! std.network - Network and socket operations module

use super::{create_external_function_named, create_function_stub};
use crate::ast::CallingConvention;
use crate::ast::{ExportStatement, Identifier, Module, PrimitiveType, TypeSpecifier};
use crate::error::SourceLocation;
use std::collections::HashMap;

/// Create the std.network module with socket operations
pub fn create_network_module() -> Module {
    let mut external_functions = HashMap::new();
    let mut functions = HashMap::new();

    // Common types
    let int_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Integer,
        source_location: SourceLocation::unknown(),
    };
    let string_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::String,
        source_location: SourceLocation::unknown(),
    };
    let bool_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Boolean,
        source_location: SourceLocation::unknown(),
    };
    let _size_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::SizeT,
        source_location: SourceLocation::unknown(),
    };

    // Socket handle type (file descriptor)
    let socket_type = int_type.clone();

    // Socket operations (external for performance)
    external_functions.insert(
        "socket_create".to_string(),
        create_external_function_named(
            "socket_create",
            "aether_socket_create",
            vec![
                ("domain", int_type.clone()),   // AF_INET = 2
                ("type", int_type.clone()),     // SOCK_STREAM = 1, SOCK_DGRAM = 2
                ("protocol", int_type.clone()), // 0 for default
            ],
            socket_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "socket_bind".to_string(),
        create_external_function_named(
            "socket_bind",
            "aether_socket_bind",
            vec![
                ("socket", socket_type.clone()),
                ("address", string_type.clone()), // IP address
                ("port", int_type.clone()),
            ],
            int_type.clone(), // 0 on success, -1 on error
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "socket_listen".to_string(),
        create_external_function_named(
            "socket_listen",
            "aether_socket_listen",
            vec![
                ("socket", socket_type.clone()),
                ("backlog", int_type.clone()),
            ],
            int_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "socket_accept".to_string(),
        create_external_function_named(
            "socket_accept",
            "aether_socket_accept",
            vec![("socket", socket_type.clone())],
            socket_type.clone(), // Returns new socket for connection
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "socket_connect".to_string(),
        create_external_function_named(
            "socket_connect",
            "aether_socket_connect",
            vec![
                ("socket", socket_type.clone()),
                ("address", string_type.clone()),
                ("port", int_type.clone()),
            ],
            int_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "socket_send".to_string(),
        create_external_function_named(
            "socket_send",
            "aether_socket_send",
            vec![
                ("socket", socket_type.clone()),
                ("data", string_type.clone()),
            ],
            int_type.clone(), // Bytes sent
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "socket_receive".to_string(),
        create_external_function_named(
            "socket_receive",
            "aether_socket_receive",
            vec![
                ("socket", socket_type.clone()),
                ("buffer_size", int_type.clone()),
            ],
            string_type.clone(), // Received data
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "socket_close".to_string(),
        create_external_function_named(
            "socket_close",
            "aether_socket_close",
            vec![("socket", socket_type.clone())],
            int_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "socket_set_option".to_string(),
        create_external_function_named(
            "socket_set_option",
            "aether_socket_set_option",
            vec![
                ("socket", socket_type.clone()),
                ("option", int_type.clone()),
                ("value", int_type.clone()),
            ],
            int_type.clone(),
            CallingConvention::C,
        ),
    );

    // High-level TCP operations (implemented in AetherScript)
    functions.insert(
        "tcp_server".to_string(),
        create_function_stub(
            "tcp_server",
            vec![("address", string_type.clone()), ("port", int_type.clone())],
            socket_type.clone(),
        ),
    );

    functions.insert(
        "tcp_client".to_string(),
        create_function_stub(
            "tcp_client",
            vec![("address", string_type.clone()), ("port", int_type.clone())],
            socket_type.clone(),
        ),
    );

    functions.insert(
        "udp_socket".to_string(),
        create_function_stub(
            "udp_socket",
            vec![("address", string_type.clone()), ("port", int_type.clone())],
            socket_type.clone(),
        ),
    );

    // HTTP-specific helpers
    functions.insert(
        "http_parse_request".to_string(),
        create_function_stub(
            "http_parse_request",
            vec![("raw_request", string_type.clone())],
            string_type.clone(), // TODO: Should return HTTP request structure
        ),
    );

    functions.insert(
        "http_create_response".to_string(),
        create_function_stub(
            "http_create_response",
            vec![
                ("status_code", int_type.clone()),
                ("headers", string_type.clone()),
                ("body", string_type.clone()),
            ],
            string_type.clone(),
        ),
    );

    functions.insert(
        "http_send_response".to_string(),
        create_function_stub(
            "http_send_response",
            vec![
                ("socket", socket_type.clone()),
                ("response", string_type.clone()),
            ],
            bool_type.clone(),
        ),
    );

    // WebSocket support
    functions.insert(
        "websocket_handshake".to_string(),
        create_function_stub(
            "websocket_handshake",
            vec![
                ("socket", socket_type.clone()),
                ("request_headers", string_type.clone()),
            ],
            bool_type.clone(),
        ),
    );

    // Network utilities
    functions.insert(
        "resolve_hostname".to_string(),
        create_function_stub(
            "resolve_hostname",
            vec![("hostname", string_type.clone())],
            string_type.clone(), // IP address
        ),
    );

    functions.insert(
        "get_local_ip".to_string(),
        create_function_stub("get_local_ip", vec![], string_type.clone()),
    );

    functions.insert(
        "is_port_available".to_string(),
        create_function_stub(
            "is_port_available",
            vec![("address", string_type.clone()), ("port", int_type.clone())],
            bool_type.clone(),
        ),
    );

    Module {
        name: Identifier::new("std.network".to_string(), SourceLocation::unknown()),
        intent: Some("Provides network socket operations and HTTP utilities for building network applications".to_string()),
        imports: vec![],
        exports: vec![
            ExportStatement::Function {
                name: Identifier::new("socket_create".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("tcp_server".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
            ExportStatement::Function {
                name: Identifier::new("http_parse_request".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            },
        ],
        type_definitions: vec![],
        trait_definitions: vec![],
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
    fn test_network_module_creation() {
        let module = create_network_module();

        assert_eq!(module.name.name, "std.network");
        assert!(module.intent.is_some());

        // Check basic socket operations
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "socket_create"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "socket_bind"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "socket_listen"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "socket_accept"));

        // Check high-level functions
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "tcp_server"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "http_parse_request"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "http_create_response"));

        // Check exports
        assert!(module.exports.iter().any(
            |e| matches!(e, ExportStatement::Function { name, .. } if name.name == "socket_create")
        ));
    }

    #[test]
    fn test_socket_function_signatures() {
        let module = create_network_module();

        // Test socket_create function
        let socket_create = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "socket_create")
            .expect("socket_create function not found");
        assert_eq!(socket_create.parameters.len(), 3);
        assert_eq!(socket_create.parameters[0].name.name, "domain");
        assert_eq!(socket_create.parameters[1].name.name, "type");
        assert_eq!(socket_create.parameters[2].name.name, "protocol");

        // Test http_create_response function
        let http_response = module
            .function_definitions
            .iter()
            .find(|f| f.name.name == "http_create_response")
            .expect("http_create_response function not found");
        assert_eq!(http_response.parameters.len(), 3);
        assert_eq!(http_response.parameters[0].name.name, "status_code");
        assert_eq!(http_response.parameters[1].name.name, "headers");
        assert_eq!(http_response.parameters[2].name.name, "body");
    }
}

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

//! AetherScript Standard Library
//!
//! Core modules providing essential functionality for AetherScript programs

pub mod collections;
pub mod console;
pub mod http;
pub mod io;
pub mod json;
pub mod math;
pub mod memory;
pub mod network;
pub mod string;

use crate::ast::{ExternalFunction, Function, Identifier, Module, Parameter, TypeSpecifier};
use crate::error::SourceLocation;
use std::collections::HashMap;

/// Standard library module registry
pub struct StandardLibrary {
    modules: HashMap<String, Module>,
}

impl StandardLibrary {
    /// Create a new standard library registry
    pub fn new() -> Self {
        let mut stdlib = Self {
            modules: HashMap::new(),
        };

        stdlib.register_core_modules();
        stdlib
    }

    /// Register all core standard library modules
    fn register_core_modules(&mut self) {
        self.register_module("std.io", io::create_io_module());
        self.register_module("std.collections", collections::create_collections_module());
        self.register_module("std.math", math::create_math_module());
        self.register_module("std.string", string::create_string_module());
        self.register_module("std.console", console::create_console_module());
        self.register_module("std.network", network::create_network_module());
        self.register_module("std.memory", memory::create_memory_module());
        self.register_module("std.http", http::create_http_module());
        self.register_module("std.json", json::create_json_module());
    }

    /// Register a standard library module
    fn register_module(&mut self, name: &str, module: Module) {
        self.modules.insert(name.to_string(), module);
    }

    /// Get a standard library module by name
    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    /// List all available standard library modules
    pub fn list_modules(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a module is a standard library module
    pub fn is_stdlib_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    /// Get all modules as a HashMap
    pub fn modules(&self) -> &HashMap<String, Module> {
        &self.modules
    }
}

impl Default for StandardLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create an external function declaration
pub(crate) fn create_external_function(
    runtime_name: &str,
    parameters: Vec<(&str, TypeSpecifier)>,
    return_type: TypeSpecifier,
    calling_convention: crate::ast::CallingConvention,
) -> ExternalFunction {
    ExternalFunction {
        name: Identifier::new(runtime_name.to_string(), SourceLocation::unknown()),
        library: "STATIC".to_string(),
        symbol: None,
        parameters: parameters
            .into_iter()
            .map(|(name, ty)| Parameter {
                name: Identifier::new(name.to_string(), SourceLocation::unknown()),
                param_type: Box::new(ty),
                intent: None,
                constraint: None,
                passing_mode: crate::ast::PassingMode::ByValue,
                source_location: SourceLocation::unknown(),
            })
            .collect(),
        return_type: Box::new(return_type),
        calling_convention,
        thread_safe: true,
        may_block: false,
        variadic: false,
        ownership_info: None,
        source_location: SourceLocation::unknown(),
    }
}

/// Helper function to create an external function with different AetherScript and runtime names
pub(crate) fn create_external_function_named(
    aether_name: &str,
    runtime_name: &str,
    parameters: Vec<(&str, TypeSpecifier)>,
    return_type: TypeSpecifier,
    calling_convention: crate::ast::CallingConvention,
) -> ExternalFunction {
    ExternalFunction {
        name: Identifier::new(aether_name.to_string(), SourceLocation::unknown()),
        library: "STATIC".to_string(),
        symbol: Some(runtime_name.to_string()),
        parameters: parameters
            .into_iter()
            .map(|(name, ty)| Parameter {
                name: Identifier::new(name.to_string(), SourceLocation::unknown()),
                param_type: Box::new(ty),
                intent: None,
                constraint: None,
                passing_mode: crate::ast::PassingMode::ByValue,
                source_location: SourceLocation::unknown(),
            })
            .collect(),
        return_type: Box::new(return_type),
        calling_convention,
        thread_safe: true,
        may_block: false,
        variadic: false,
        ownership_info: None,
        source_location: SourceLocation::unknown(),
    }
}

/// Helper function to create a function definition with minimal structure
pub(crate) fn create_function_stub(
    name: &str,
    parameters: Vec<(&str, TypeSpecifier)>,
    return_type: TypeSpecifier,
) -> Function {
    Function {
        name: Identifier::new(name.to_string(), SourceLocation::unknown()),
        intent: None,
        generic_parameters: Vec::new(),
        lifetime_parameters: Vec::new(),
        parameters: parameters
            .into_iter()
            .map(|(name, ty)| Parameter {
                name: Identifier::new(name.to_string(), SourceLocation::unknown()),
                param_type: Box::new(ty),
                intent: None,
                constraint: None,
                passing_mode: crate::ast::PassingMode::ByValue,
                source_location: SourceLocation::unknown(),
            })
            .collect(),
        return_type: Box::new(return_type),
        metadata: crate::ast::FunctionMetadata {
            preconditions: vec![],
            postconditions: vec![],
            invariants: vec![],
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: vec![],
            thread_safe: None,
            may_block: None,
        },
        body: crate::ast::Block {
            statements: vec![], // Empty body - would be filled in by actual implementation
            source_location: SourceLocation::unknown(),
        },
        export_info: None,
        is_async: false,
        source_location: SourceLocation::unknown(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_creation() {
        let stdlib = StandardLibrary::new();

        // Check that all expected modules are registered
        assert!(stdlib.is_stdlib_module("std.io"));
        assert!(stdlib.is_stdlib_module("std.collections"));
        assert!(stdlib.is_stdlib_module("std.math"));
        assert!(stdlib.is_stdlib_module("std.string"));
        assert!(stdlib.is_stdlib_module("std.console"));
        assert!(stdlib.is_stdlib_module("std.network"));
        assert!(stdlib.is_stdlib_module("std.memory"));
        assert!(stdlib.is_stdlib_module("std.http"));
        assert!(stdlib.is_stdlib_module("std.json"));

        // Check that non-existent modules return false
        assert!(!stdlib.is_stdlib_module("std.nonexistent"));
    }

    #[test]
    fn test_module_listing() {
        let stdlib = StandardLibrary::new();
        let modules = stdlib.list_modules();

        assert_eq!(modules.len(), 9);
        assert!(modules.contains(&"std.io"));
        assert!(modules.contains(&"std.collections"));
        assert!(modules.contains(&"std.math"));
        assert!(modules.contains(&"std.string"));
        assert!(modules.contains(&"std.console"));
        assert!(modules.contains(&"std.network"));
        assert!(modules.contains(&"std.memory"));
        assert!(modules.contains(&"std.http"));
        assert!(modules.contains(&"std.json"));
    }

    #[test]
    fn test_module_retrieval() {
        let stdlib = StandardLibrary::new();

        // Test retrieving existing modules
        assert!(stdlib.get_module("std.io").is_some());
        assert!(stdlib.get_module("std.math").is_some());

        // Test retrieving non-existent module
        assert!(stdlib.get_module("std.nonexistent").is_none());
    }
}

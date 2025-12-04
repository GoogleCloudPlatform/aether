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

//! Module loader for AetherScript
//!
//! Responsible for finding, loading, and caching modules from various sources.
//! Supports loading from:
//! - Source files (.aether)
//! - Pre-compiled ABI files (.abi) for separate compilation
//! - Standard library (embedded or from stdlib/)
//! - In-memory (for testing)

use crate::abi::AbiModule;
use crate::ast::{self, Module};
use crate::error::{SemanticError, SourceLocation};
use crate::lexer::v2::Lexer;
use crate::parser::v2::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Source of a module
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleSource {
    /// File system path
    File(PathBuf),
    /// Pre-compiled ABI file (for separate compilation)
    Abi(PathBuf),
    /// Standard library module
    Stdlib(String),
    /// Package module
    Package(String, String), // (package_name, module_name)
    /// In-memory module (for testing)
    Memory(String), // source code
}

/// Loaded module information
#[derive(Debug, Clone)]
pub struct LoadedModule {
    pub module: Module,
    pub source: ModuleSource,
    pub dependencies: Vec<String>,
    /// If true, this module was loaded from an ABI file (pre-compiled)
    /// and doesn't have function bodies - only signatures
    pub from_abi: bool,
    /// Path to the object file (if loaded from ABI)
    pub object_file: Option<PathBuf>,
}

/// Module loader that handles module resolution and caching
pub struct ModuleLoader {
    /// Cache of loaded modules
    module_cache: HashMap<String, LoadedModule>,

    /// Search paths for modules
    search_paths: Vec<PathBuf>,

    /// Standard library modules (module name -> source code)
    stdlib_modules: HashMap<String, String>,
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleLoader {
    pub fn new() -> Self {
        let mut loader = Self {
            module_cache: HashMap::new(),
            search_paths: vec![
                PathBuf::from("."),
                PathBuf::from("./modules"),
                PathBuf::from("./src"),
            ],
            stdlib_modules: HashMap::new(),
        };

        // Register standard library modules
        loader.register_stdlib_modules();
        loader
    }

    /// Add a search path for modules
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Load a module by name
    pub fn load_module(&mut self, module_name: &str) -> Result<&LoadedModule, SemanticError> {
        // Check cache first
        if self.module_cache.contains_key(module_name) {
            return Ok(&self.module_cache[module_name]);
        }

        // Try to resolve and load the module
        let source = self.resolve_module(module_name)?;

        // Handle ABI vs source loading
        let (module, from_abi, object_file) = match &source {
            ModuleSource::Abi(abi_path) => {
                let abi = self.load_abi_file(abi_path)?;
                let module = self.abi_to_module(&abi)?;
                // Object file is the .o file next to the .abi file
                let obj_path = abi_path.with_extension("o");
                let object_file = if obj_path.exists() { Some(obj_path) } else { None };
                (module, true, object_file)
            }
            _ => {
                let module = self.parse_module(module_name, &source)?;
                (module, false, None)
            }
        };

        // Extract dependencies
        let dependencies: Vec<String> = module
            .imports
            .iter()
            .map(|import| import.module_name.name.clone())
            .collect();

        // Cache the loaded module
        let loaded = LoadedModule {
            module,
            source,
            dependencies,
            from_abi,
            object_file,
        };

        self.module_cache.insert(module_name.to_string(), loaded);
        Ok(&self.module_cache[module_name])
    }

    /// Resolve a module name to its source
    fn resolve_module(&self, module_name: &str) -> Result<ModuleSource, SemanticError> {
        // 1. Check if it's a standard library module (using underscore convention)
        if module_name.starts_with("std_") {
            let stdlib_name = module_name.strip_prefix("std_").unwrap();
            if self.stdlib_modules.contains_key(stdlib_name) {
                return Ok(ModuleSource::Stdlib(stdlib_name.to_string()));
            }
        }

        // Also check with dot notation for backward compatibility
        if module_name.starts_with("std.") {
            let stdlib_name = module_name.strip_prefix("std.").unwrap();
            if self.stdlib_modules.contains_key(stdlib_name) {
                return Ok(ModuleSource::Stdlib(stdlib_name.to_string()));
            }
        }

        // 2. Check file system paths
        // Try multiple filename variations to support different naming conventions
        let exact_name = module_name.replace('.', "/");
        let snake_name = to_snake_case(&exact_name);
        let pascal_name = to_pascal_case(&exact_name);

        // Prefer exact match, then snake_case, then PascalCase
        let mut variants = vec![exact_name.clone()];
        if snake_name != exact_name {
            variants.push(snake_name);
        }
        if pascal_name != exact_name {
            variants.push(pascal_name);
        }

        // Check for ABI files FIRST (prefer pre-compiled over source)
        for name in &variants {
            let abi_filename = format!("{}.abi", name);
            for search_path in &self.search_paths {
                let abi_path = search_path.join(&abi_filename);
                if abi_path.exists() {
                    return Ok(ModuleSource::Abi(abi_path));
                }
            }
        }

        // Then check for source files
        for name in &variants {
            let module_filename = format!("{}.aether", name);
            for search_path in &self.search_paths {
                let full_path = search_path.join(&module_filename);
                if full_path.exists() {
                    return Ok(ModuleSource::File(full_path));
                }
            }
        }

        // 3. Check packages (TODO: integrate with package system)
        if module_name.contains("::") {
            let parts: Vec<&str> = module_name.split("::").collect();
            if parts.len() == 2 {
                // For now, return error - package integration not complete
                return Err(SemanticError::Internal {
                    message: format!(
                        "Package module resolution not yet implemented: {}",
                        module_name
                    ),
                });
            }
        }

        Err(SemanticError::Internal {
            message: format!(
                "Module '{}' not found in search paths: {:?}",
                module_name, self.search_paths
            ),
        })
    }

    /// Load an ABI file
    fn load_abi_file(&self, path: &PathBuf) -> Result<AbiModule, SemanticError> {
        AbiModule::load(path).map_err(|e| SemanticError::IoError {
            message: format!("Failed to load ABI file '{}': {}", path.display(), e),
        })
    }

    /// Convert an ABI module to an AST Module
    fn abi_to_module(&self, abi: &AbiModule) -> Result<Module, SemanticError> {

        // Create a synthetic module with the interface from the ABI
        let mut external_functions = Vec::new();
        let mut exports = Vec::new();

        for func in &abi.functions {
            let ext_func = self.abi_func_to_external_function(func)?;
            external_functions.push(ext_func);

            // Add all functions to exports (they're public in the ABI)
            exports.push(ast::ExportStatement::Function {
                name: ast::Identifier {
                    name: func.name.clone(),
                    source_location: SourceLocation::unknown(),
                },
                source_location: SourceLocation::unknown(),
            });
        }

        // Create type definitions from ABI types
        let mut type_definitions = Vec::new();
        for abi_struct in &abi.types.structs {
            let type_def = self.abi_struct_to_type_def(abi_struct)?;

            // Export the struct type
            if let ast::TypeDefinition::Structured { ref name, .. } = type_def {
                exports.push(ast::ExportStatement::Type {
                    name: name.clone(),
                    source_location: SourceLocation::unknown(),
                });
            }

            type_definitions.push(type_def);
        }

        for abi_enum in &abi.types.enums {
            let type_def = self.abi_enum_to_type_def(abi_enum)?;

            // Export the enum type
            if let ast::TypeDefinition::Enumeration { ref name, .. } = type_def {
                exports.push(ast::ExportStatement::Type {
                    name: name.clone(),
                    source_location: SourceLocation::unknown(),
                });
            }

            type_definitions.push(type_def);
        }

        Ok(Module {
            name: ast::Identifier {
                name: abi.module.name.clone(),
                source_location: SourceLocation::unknown(),
            },
            intent: None,
            imports: Vec::new(), // Dependencies are tracked separately
            exports,
            type_definitions,
            trait_definitions: Vec::new(), // TODO: Convert traits from ABI
            impl_blocks: Vec::new(),
            constant_declarations: Vec::new(),
            function_definitions: Vec::new(), // Native functions are in external_functions
            external_functions,
            source_location: SourceLocation::unknown(),
        })
    }

    /// Convert an ABI function to an ExternalFunction AST node
    fn abi_func_to_external_function(
        &self,
        func: &crate::abi::AbiFunction,
    ) -> Result<ast::ExternalFunction, SemanticError> {
        use crate::abi::FunctionKind;

        let (library, symbol) = match &func.kind {
            FunctionKind::Extern { library, symbol, .. } => {
                (library.clone(), Some(symbol.clone()))
            }
            FunctionKind::Native { symbol } => {
                // Use special marker library "__abi__" for native functions from pre-compiled modules
                // This tells the FFI analyzer to skip library validation
                ("__abi__".to_string(), Some(symbol.clone()))
            }
            FunctionKind::Generic { symbol_prefix, .. } => {
                ("__abi__".to_string(), Some(symbol_prefix.clone()))
            }
        };

        let parameters: Vec<ast::Parameter> = func.signature.parameters
            .iter()
            .map(|p| ast::Parameter {
                name: ast::Identifier {
                    name: p.name.clone(),
                    source_location: SourceLocation::unknown(),
                },
                param_type: Box::new(self.abi_type_to_type_specifier(&p.ty)),
                intent: None,
                constraint: None,
                passing_mode: ast::PassingMode::ByValue,
                source_location: SourceLocation::unknown(),
            })
            .collect();

        Ok(ast::ExternalFunction {
            name: ast::Identifier {
                name: func.name.clone(),
                source_location: SourceLocation::unknown(),
            },
            library,
            symbol,
            parameters,
            return_type: Box::new(self.abi_type_to_type_specifier(&func.signature.return_type)),
            calling_convention: ast::CallingConvention::C,
            thread_safe: true,
            may_block: false,
            variadic: func.signature.is_variadic,
            ownership_info: None,
            source_location: SourceLocation::unknown(),
        })
    }

    /// Convert an ABI type to a TypeSpecifier
    fn abi_type_to_type_specifier(&self, abi_type: &crate::abi::AbiType) -> ast::TypeSpecifier {
        use crate::abi::AbiType;

        match abi_type {
            AbiType::Primitive { name } => ast::TypeSpecifier::Primitive {
                type_name: match name.as_str() {
                    "Int" => ast::PrimitiveType::Integer,
                    "Int32" => ast::PrimitiveType::Integer32,
                    "Int64" => ast::PrimitiveType::Integer64,
                    "Float" => ast::PrimitiveType::Float,
                    "Float32" => ast::PrimitiveType::Float32,
                    "Float64" => ast::PrimitiveType::Float64,
                    "Bool" => ast::PrimitiveType::Boolean,
                    "String" => ast::PrimitiveType::String,
                    "Char" => ast::PrimitiveType::Char,
                    "Void" => ast::PrimitiveType::Void,
                    "SizeT" => ast::PrimitiveType::SizeT,
                    "UIntPtrT" => ast::PrimitiveType::UIntPtrT,
                    _ => ast::PrimitiveType::Void,
                },
                source_location: SourceLocation::unknown(),
            },
            AbiType::Named { name, .. } => ast::TypeSpecifier::Named {
                name: ast::Identifier {
                    name: name.clone(),
                    source_location: SourceLocation::unknown(),
                },
                source_location: SourceLocation::unknown(),
            },
            AbiType::Array { element, size } => ast::TypeSpecifier::Array {
                element_type: Box::new(self.abi_type_to_type_specifier(element)),
                size: size.map(|s| Box::new(ast::Expression::IntegerLiteral {
                    value: s as i64,
                    source_location: SourceLocation::unknown(),
                })),
                source_location: SourceLocation::unknown(),
            },
            AbiType::Pointer { target, mutable } => ast::TypeSpecifier::Pointer {
                target_type: Box::new(self.abi_type_to_type_specifier(target)),
                is_mutable: *mutable,
                source_location: SourceLocation::unknown(),
            },
            AbiType::GenericInstance { base, args, .. } => ast::TypeSpecifier::Generic {
                base_type: ast::Identifier {
                    name: base.clone(),
                    source_location: SourceLocation::unknown(),
                },
                type_arguments: args.iter()
                    .map(|a| Box::new(self.abi_type_to_type_specifier(a)))
                    .collect(),
                source_location: SourceLocation::unknown(),
            },
            AbiType::GenericParam { name } => ast::TypeSpecifier::TypeParameter {
                name: ast::Identifier {
                    name: name.clone(),
                    source_location: SourceLocation::unknown(),
                },
                constraints: Vec::new(),
                source_location: SourceLocation::unknown(),
            },
            AbiType::Unit => ast::TypeSpecifier::Primitive {
                type_name: ast::PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            },
            _ => ast::TypeSpecifier::Primitive {
                type_name: ast::PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            },
        }
    }

    /// Convert an ABI struct to a TypeDefinition
    fn abi_struct_to_type_def(
        &self,
        abi_struct: &crate::abi::AbiStruct,
    ) -> Result<ast::TypeDefinition, SemanticError> {
        let fields: Vec<ast::StructField> = abi_struct.fields
            .iter()
            .map(|f| ast::StructField {
                name: ast::Identifier {
                    name: f.name.clone(),
                    source_location: SourceLocation::unknown(),
                },
                field_type: Box::new(self.abi_type_to_type_specifier(&f.ty)),
                source_location: SourceLocation::unknown(),
            })
            .collect();

        Ok(ast::TypeDefinition::Structured {
            name: ast::Identifier {
                name: abi_struct.name.clone(),
                source_location: SourceLocation::unknown(),
            },
            intent: None,
            generic_parameters: Vec::new(), // TODO: Convert generics
            lifetime_parameters: Vec::new(),
            where_clause: Vec::new(),
            fields,
            export_as: None,
            source_location: SourceLocation::unknown(),
        })
    }

    /// Convert an ABI enum to a TypeDefinition
    fn abi_enum_to_type_def(
        &self,
        abi_enum: &crate::abi::AbiEnum,
    ) -> Result<ast::TypeDefinition, SemanticError> {
        let variants: Vec<ast::EnumVariant> = abi_enum.variants
            .iter()
            .map(|v| ast::EnumVariant {
                name: ast::Identifier {
                    name: v.name.clone(),
                    source_location: SourceLocation::unknown(),
                },
                associated_types: v.fields
                    .iter()
                    .map(|f| self.abi_type_to_type_specifier(&f.ty))
                    .collect(),
                source_location: SourceLocation::unknown(),
            })
            .collect();

        Ok(ast::TypeDefinition::Enumeration {
            name: ast::Identifier {
                name: abi_enum.name.clone(),
                source_location: SourceLocation::unknown(),
            },
            intent: None,
            generic_parameters: Vec::new(),
            lifetime_parameters: Vec::new(),
            where_clause: Vec::new(),
            variants,
            source_location: SourceLocation::unknown(),
        })
    }

    /// Parse a module from its source
    fn parse_module(
        &self,
        module_name: &str,
        source: &ModuleSource,
    ) -> Result<Module, SemanticError> {
        let source_code = match source {
            ModuleSource::File(path) => {
                fs::read_to_string(path).map_err(|e| SemanticError::IoError {
                    message: format!("Failed to read module file '{}': {}", path.display(), e),
                })?
            }
            ModuleSource::Abi(_) => {
                // ABI sources are handled by abi_to_module, not parse_module
                return Err(SemanticError::Internal {
                    message: "ABI modules should be loaded via abi_to_module, not parse_module".to_string(),
                });
            }
            ModuleSource::Stdlib(name) => self
                .stdlib_modules
                .get(name)
                .ok_or_else(|| SemanticError::Internal {
                    message: format!("Standard library module '{}' not found", name),
                })?
                .clone(),
            ModuleSource::Package(_, _) => {
                return Err(SemanticError::Internal {
                    message: "Package module loading not yet implemented".to_string(),
                });
            }
            ModuleSource::Memory(code) => code.clone(),
        };

        // Tokenize and parse the module using V2 lexer/parser
        let mut lexer = Lexer::new(&source_code, module_name.to_string());
        let tokens = lexer.tokenize().map_err(|e| SemanticError::Internal {
            message: format!("Failed to tokenize module '{}': {}", module_name, e),
        })?;

        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().map_err(|e| SemanticError::Internal {
            message: format!("Failed to parse module '{}': {}", module_name, e),
        })?;

        Ok(module)
    }

    /// Register standard library modules
    fn register_stdlib_modules(&mut self) {
        // For now, we'll register the stdlib modules as empty - in a real implementation
        // these would be loaded from embedded files or a stdlib directory

        // Core module
        self.stdlib_modules.insert(
            "core".to_string(),
            std::fs::read_to_string("stdlib/core.aether").unwrap_or_else(|_| String::new()),
        );

        // I/O module
        self.stdlib_modules.insert(
            "io".to_string(),
            std::fs::read_to_string("stdlib/io.aether").unwrap_or_else(|_| String::new()),
        );

        // Math module
        self.stdlib_modules.insert(
            "math".to_string(),
            std::fs::read_to_string("stdlib/math.aether").unwrap_or_else(|_| String::new()),
        );

        // Collections module
        self.stdlib_modules.insert(
            "collections".to_string(),
            std::fs::read_to_string("stdlib/collections.aether")
                .unwrap_or_else(|_| String::new()),
        );

        // String utilities
        self.stdlib_modules.insert(
            "string".to_string(),
            std::fs::read_to_string("stdlib/string.aether").unwrap_or_else(|_| String::new()),
        );

        // StringView - zero-copy string operations
        self.stdlib_modules.insert(
            "stringview".to_string(),
            std::fs::read_to_string("stdlib/stringview.aether").unwrap_or_else(|_| String::new()),
        );
    }

    /// Get all loaded modules
    pub fn loaded_modules(&self) -> &HashMap<String, LoadedModule> {
        &self.module_cache
    }

    /// Check for circular dependencies
    pub fn check_circular_dependencies(&self, module_name: &str) -> Result<(), SemanticError> {
        let mut visited = HashMap::new();
        let mut stack = Vec::new();

        self.check_circular_deps_recursive(module_name, &mut visited, &mut stack)
    }

    fn check_circular_deps_recursive(
        &self,
        module_name: &str,
        visited: &mut HashMap<String, bool>,
        stack: &mut Vec<String>,
    ) -> Result<(), SemanticError> {
        // If we're already in the stack, we have a cycle
        if stack.contains(&module_name.to_string()) {
            return Err(SemanticError::CircularDependency {
                module: module_name.to_string(),
                location: SourceLocation::unknown(),
            });
        }

        // If already fully visited, no cycle through this node
        if visited.get(module_name) == Some(&true) {
            return Ok(());
        }

        // Mark as being visited
        stack.push(module_name.to_string());

        // Visit dependencies
        if let Some(loaded) = self.module_cache.get(module_name) {
            for dep in &loaded.dependencies {
                self.check_circular_deps_recursive(dep, visited, stack)?;
            }
        }

        // Mark as fully visited
        stack.pop();
        visited.insert(module_name.to_string(), true);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_module_loading() {
        let mut loader = ModuleLoader::new();

        // Test loading a stdlib module
        let result = loader.resolve_module("std.core");
        assert!(result.is_ok());
        match result.unwrap() {
            ModuleSource::Stdlib(name) => assert_eq!(name, "core"),
            _ => panic!("Expected stdlib module"),
        }
    }

    #[test]
    fn test_abi_loading() {
        use std::io::Write;
        use tempfile::TempDir;

        let loader = ModuleLoader::new();

        // Create a temporary directory with an ABI file
        let temp_dir = TempDir::new().unwrap();
        let abi_path = temp_dir.path().join("test_module.abi");

        // Create a simple ABI JSON file
        let abi_json = r#"{
            "abi_version": "1.0.0",
            "aether_version": "0.1.0",
            "module": {
                "name": "TestModule",
                "path": "test_module.aether",
                "checksum": null
            },
            "dependencies": [],
            "functions": [
                {
                    "name": "add_numbers",
                    "signature": {
                        "generic_params": [],
                        "where_clauses": [],
                        "parameters": [
                            {
                                "name": "a",
                                "ty": { "kind": "Primitive", "name": "Int" },
                                "mode": "Owned"
                            },
                            {
                                "name": "b",
                                "ty": { "kind": "Primitive", "name": "Int" },
                                "mode": "Owned"
                            }
                        ],
                        "return_type": { "kind": "Primitive", "name": "Int" },
                        "is_variadic": false
                    },
                    "kind": { "type": "Extern", "library": "test_lib", "symbol": "add_numbers", "calling_convention": "C" },
                    "contracts": { "preconditions": [], "postconditions": [], "verified": false, "assumes_axioms": [] },
                    "attributes": ["extern"],
                    "source_location": { "line": 1, "column": 1 },
                    "is_public": true
                }
            ],
            "types": { "structs": [], "enums": [], "type_aliases": [] },
            "traits": [],
            "constants": [],
            "impls": []
        }"#;

        std::fs::File::create(&abi_path)
            .unwrap()
            .write_all(abi_json.as_bytes())
            .unwrap();

        // Test loading the ABI file
        let result = loader.load_abi_file(&abi_path);
        assert!(result.is_ok(), "Failed to load ABI: {:?}", result.err());

        let abi_module = result.unwrap();
        assert_eq!(abi_module.module.name, "TestModule");
        assert_eq!(abi_module.functions.len(), 1);
        assert_eq!(abi_module.functions[0].name, "add_numbers");

        // Test converting ABI to Module
        let module_result = loader.abi_to_module(&abi_module);
        assert!(
            module_result.is_ok(),
            "Failed to convert ABI to Module: {:?}",
            module_result.err()
        );

        let module = module_result.unwrap();
        assert_eq!(module.name.name, "TestModule");
        assert_eq!(module.external_functions.len(), 1);
        assert_eq!(module.external_functions[0].name.name, "add_numbers");
    }

    #[test]
    fn test_module_caching() {
        let mut loader = ModuleLoader::new();

        // Add in-memory test module
        loader.module_cache.insert(
            "test".to_string(),
            LoadedModule {
                module: Module {
                    name: crate::ast::Identifier {
                        name: "test".to_string(),
                        source_location: SourceLocation::unknown(),
                    },
                    intent: None,
                    imports: vec![],
                    exports: vec![],
                    type_definitions: vec![],
                    trait_definitions: vec![],
                    impl_blocks: vec![],
                    constant_declarations: vec![],
                    function_definitions: vec![],
                    external_functions: vec![],
                    source_location: SourceLocation::unknown(),
                },
                source: ModuleSource::Memory("test module".to_string()),
                dependencies: vec![],
                from_abi: false,
                object_file: None,
            },
        );

        // Loading same module twice should use cache
        let result1 = loader.load_module("test");
        assert!(result1.is_ok());

        // Load again - should use cache
        let result2 = loader.load_module("test");
        assert!(result2.is_ok());

        // Both loads should succeed and return cached module
        // We can't directly compare pointers due to borrow checker,
        // but we know caching works if both loads succeed
    }
}

/// Convert string to snake_case (e.g., "MyModule" -> "my_module")
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0
                && s.chars()
                    .nth(i - 1)
                    .map(|p| !p.is_uppercase())
                    .unwrap_or(true)
            {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert string to PascalCase (e.g., "my_module" -> "MyModule")
fn to_pascal_case(s: &str) -> String {
    s.split(['_', '/'])
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

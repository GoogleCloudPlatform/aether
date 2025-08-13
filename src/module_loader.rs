//! Module loader for AetherScript
//! 
//! Responsible for finding, loading, and caching modules from various sources

use crate::ast::Module;
use crate::parser::Parser;
use crate::error::{SemanticError, SourceLocation};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

/// Source of a module
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleSource {
    /// File system path
    File(PathBuf),
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
        let module = self.parse_module(module_name, &source)?;
        
        // Extract dependencies
        let dependencies: Vec<String> = module.imports.iter()
            .map(|import| import.module_name.name.clone())
            .collect();
        
        // Cache the loaded module
        let loaded = LoadedModule {
            module,
            source,
            dependencies,
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
        let module_filename = format!("{}.aether", module_name.replace('.', "/"));
        
        for search_path in &self.search_paths {
            let full_path = search_path.join(&module_filename);
            if full_path.exists() {
                return Ok(ModuleSource::File(full_path));
            }
        }
        
        // 3. Check packages (TODO: integrate with package system)
        if module_name.contains("::") {
            let parts: Vec<&str> = module_name.split("::").collect();
            if parts.len() == 2 {
                // For now, return error - package integration not complete
                return Err(SemanticError::Internal {
                    message: format!("Package module resolution not yet implemented: {}", module_name),
                });
            }
        }
        
        Err(SemanticError::Internal {
            message: format!("Module '{}' not found in search paths: {:?}", module_name, self.search_paths),
        })
    }
    
    /// Parse a module from its source
    fn parse_module(&self, module_name: &str, source: &ModuleSource) -> Result<Module, SemanticError> {
        let source_code = match source {
            ModuleSource::File(path) => {
                fs::read_to_string(path)
                    .map_err(|e| SemanticError::IoError {
                        message: format!("Failed to read module file '{}': {}", path.display(), e),
                    })?
            }
            ModuleSource::Stdlib(name) => {
                self.stdlib_modules.get(name)
                    .ok_or_else(|| SemanticError::Internal {
                        message: format!("Standard library module '{}' not found", name),
                    })?
                    .clone()
            }
            ModuleSource::Package(_, _) => {
                return Err(SemanticError::Internal {
                    message: "Package module loading not yet implemented".to_string(),
                });
            }
            ModuleSource::Memory(code) => code.clone(),
        };
        
        // Tokenize and parse the module
        let mut lexer = crate::lexer::Lexer::new(&source_code, module_name.to_string());
        let tokens = lexer.tokenize()
            .map_err(|e| SemanticError::Internal {
                message: format!("Failed to tokenize module '{}': {}", module_name, e),
            })?;
            
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()
            .map_err(|e| SemanticError::Internal {
                message: format!("Failed to parse module '{}': {}", module_name, e),
            })?;
        
        // Extract the module (assuming single-module files for now)
        if program.modules.len() != 1 {
            return Err(SemanticError::Internal {
                message: format!("Module file '{}' must contain exactly one module", module_name),
            });
        }
        
        Ok(program.modules.into_iter().next().unwrap())
    }
    
    /// Register standard library modules
    fn register_stdlib_modules(&mut self) {
        // For now, we'll register the stdlib modules as empty - in a real implementation
        // these would be loaded from embedded files or a stdlib directory
        
        // Core module
        self.stdlib_modules.insert("core".to_string(), 
            std::fs::read_to_string("src/stdlib/core.aether").unwrap_or_else(|_| String::new()));
        
        // I/O module
        self.stdlib_modules.insert("io".to_string(), 
            std::fs::read_to_string("src/stdlib/io.aether").unwrap_or_else(|_| String::new()));
        
        // Math module
        self.stdlib_modules.insert("math".to_string(), 
            std::fs::read_to_string("src/stdlib/math.aether").unwrap_or_else(|_| String::new()));
        
        // Collections module
        self.stdlib_modules.insert("collections".to_string(), 
            std::fs::read_to_string("src/stdlib/collections.aether").unwrap_or_else(|_| String::new()));
        
        // String utilities
        self.stdlib_modules.insert("string".to_string(), 
            std::fs::read_to_string("src/stdlib/string.aether").unwrap_or_else(|_| String::new()));
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
            let cycle_start = stack.iter().position(|m| m == module_name).unwrap();
            let cycle: Vec<String> = stack[cycle_start..].to_vec();
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
    fn test_module_caching() {
        let mut loader = ModuleLoader::new();
        
        // Add in-memory test module
        loader.module_cache.insert("test".to_string(), LoadedModule {
            module: Module {
                name: crate::ast::Identifier {
                    name: "test".to_string(),
                    source_location: SourceLocation::unknown(),
                },
                intent: None,
                imports: vec![],
                exports: vec![],
                type_definitions: vec![],
                constant_declarations: vec![],
                function_definitions: vec![],
                external_functions: vec![],
                source_location: SourceLocation::unknown(),
            },
            source: ModuleSource::Memory("test module".to_string()),
            dependencies: vec![],
        });
        
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
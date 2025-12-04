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

//! Semantic analysis for AetherScript
//!
//! Performs type checking, symbol resolution, and semantic validation

pub mod capture_analysis;
pub mod metadata;
pub mod ownership;
#[cfg(test)]
mod ownership_tests;

use crate::ast::*;
use crate::contracts::{ContractContext, ContractValidator};
use crate::error::{SemanticError, SourceLocation};
use crate::ffi::FFIAnalyzer;
use crate::memory::MemoryAnalyzer;
use crate::module_loader::{LoadedModule, ModuleLoader};
use crate::symbols::{BorrowState, ScopeKind, Symbol, SymbolKind, SymbolTable};
use crate::types::{Type, TypeChecker};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Semantic analyzer for AetherScript programs
pub struct SemanticAnalyzer {
    /// Symbol table for variable and type tracking
    symbol_table: SymbolTable,

    /// Type checker for type inference and compatibility
    type_checker: Rc<RefCell<TypeChecker>>,

    /// Contract validator for metadata and contract checking
    contract_validator: ContractValidator,

    /// FFI analyzer for external function declarations
    ffi_analyzer: FFIAnalyzer,

    /// Expected return type of the function currently being analyzed
    current_function_return_type: Option<Type>,

    /// Memory analyzer for deterministic memory management
    memory_analyzer: MemoryAnalyzer,

    /// Module loader for resolving imports
    module_loader: ModuleLoader,

    /// Current module being analyzed
    current_module: Option<String>,

    /// Errors collected during analysis
    errors: Vec<SemanticError>,

    /// Analysis statistics
    stats: AnalysisStats,

    /// Exception types that can be thrown in current context
    current_exceptions: Vec<Type>,

    /// Whether we're currently in a finally block (affects throw analysis)
    in_finally_block: bool,

    /// Whether we're currently in a concurrent block (affects function return types)
    in_concurrent_block: bool,

    /// Analyzed modules cache to prevent double-analysis
    analyzed_modules: HashMap<String, LoadedModule>,

    /// Capture analysis results for concurrent blocks
    pub captures: HashMap<SourceLocation, std::collections::HashSet<String>>,

    /// Registered trait definitions (by simple name)
    trait_definitions: HashMap<String, TraitDefinition>,

    /// Dispatch table for trait method calls keyed by receiver type and method name
    trait_dispatch_table: HashMap<TraitMethodKey, TraitMethodDispatch>,

    /// Constraints for generic parameters in the current function (from where clauses)
    current_generic_constraints: HashMap<String, Vec<crate::types::TypeConstraintInfo>>,
}

/// Statistics about the semantic analysis
#[derive(Debug, Clone, Default)]
pub struct AnalysisStats {
    pub modules_analyzed: usize,
    pub functions_analyzed: usize,
    pub variables_declared: usize,
    pub types_defined: usize,
    pub external_functions_analyzed: usize,
    pub errors_found: usize,
}

/// Key for trait method dispatch lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitMethodKey {
    pub receiver: Type,
    pub method_name: String,
}

/// Dispatch information for a resolved trait method
#[derive(Debug, Clone)]
pub struct TraitMethodDispatch {
    pub trait_name: String,
    pub impl_type: Type,
    pub method_name: String,
    pub self_param_type: Option<Type>,
    pub param_types: Vec<Type>,
    pub return_type: Type,
    pub symbol_name: String,
}

/// Internal representation of a method signature after substituting `Self` and generics
#[derive(Debug, Clone)]
struct MethodSignatureInfo {
    self_param_type: Option<Type>,
    call_param_types: Vec<Type>,
    all_param_types: Vec<Type>,
    return_type: Type,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        eprintln!("SemanticAnalyzer: Creating new instance");
        let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
        let ffi_analyzer = FFIAnalyzer::new(type_checker.clone());
        let memory_analyzer = MemoryAnalyzer::new(type_checker.clone());

        Self {
            symbol_table: SymbolTable::new(),
            type_checker,
            contract_validator: ContractValidator::new(),
            ffi_analyzer,
            memory_analyzer,
            module_loader: ModuleLoader::new(),
            current_module: None,
            errors: Vec::new(),
            stats: AnalysisStats::default(),
            current_exceptions: Vec::new(),
            in_finally_block: false,
            in_concurrent_block: false,
            analyzed_modules: HashMap::new(),
            captures: HashMap::new(),
            current_function_return_type: None,
            trait_definitions: HashMap::new(),
            trait_dispatch_table: HashMap::new(),
            current_generic_constraints: HashMap::new(),
        }
    }

    /// Add a search path for modules
    pub fn add_module_search_path(&mut self, path: std::path::PathBuf) {
        self.module_loader.add_search_path(path);
    }

    /// Analyze a complete program
    pub fn analyze_program(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        self.errors.clear();

        for module in &program.modules {
            if let Err(e) = self.analyze_module(module) {
                self.errors.push(e);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Analyze a module
    pub fn analyze_module(&mut self, module: &Module) -> Result<(), SemanticError> {
        // Check if already analyzed
        if self.analyzed_modules.contains_key(&module.name.name) {
            return Ok(());
        }

        let prev_sa_module = self.current_module.clone();
        let prev_tc_module = self.type_checker.borrow().get_current_module();

        self.current_module = Some(module.name.name.clone());
        self.symbol_table
            .set_current_module(self.current_module.clone());
        self.type_checker
            .borrow_mut()
            .set_current_module(self.current_module.clone());

        // Create and enter a root memory region for the module
        let root_region = self.memory_analyzer.create_region(None);
        self.memory_analyzer.enter_region(root_region);

        // Enter module scope
        self.symbol_table.enter_scope(ScopeKind::Module);

        // Process imports first
        for import in &module.imports {
            self.analyze_import(import)?;
        }

        // Process type definitions
        for type_def in &module.type_definitions {
            self.analyze_type_definition(type_def)?;
        }

        // Register trait definitions
        for trait_def in &module.trait_definitions {
            self.analyze_trait_definition(trait_def)?;
        }

        // Process impl blocks (both trait and inherent)
        for impl_block in &module.impl_blocks {
            self.analyze_impl_block(impl_block)?;
        }

        // Process constant declarations
        for const_decl in &module.constant_declarations {
            self.analyze_constant_declaration(const_decl)?;
        }

        // Process external function declarations BEFORE regular functions
        // so that regular functions can call external functions
        for ext_func in &module.external_functions {
            self.analyze_external_function(ext_func)?;
        }

        // First pass: Add all function signatures to symbol table
        for func_def in &module.function_definitions {
            self.add_function_signature(func_def)?;
        }

        // Cache the module BEFORE analyzing function bodies
        // This allows function body analysis to look up generic parameters of
        // other functions in the same module
        let dependencies: Vec<String> = module
            .imports
            .iter()
            .map(|import| import.module_name.name.clone())
            .collect();
        let loaded_module = LoadedModule {
            module: module.clone(),
            source: crate::module_loader::ModuleSource::Memory("".to_string()),
            dependencies,
            from_abi: false,
            object_file: None,
        };
        self.analyzed_modules
            .insert(module.name.name.clone(), loaded_module);

        // Second pass: Analyze function bodies
        for func_def in &module.function_definitions {
            self.analyze_function_body(func_def)?;
        }

        // Process exports (validate that exported symbols exist)
        for export in &module.exports {
            self.analyze_export(export)?;
        }

        // Run capture analysis
        let mut capture_analyzer = capture_analysis::CaptureAnalyzer::new();
        capture_analyzer.analyze(module);
        self.captures.extend(capture_analyzer.captures);

        // Exit module scope
        self.symbol_table.exit_scope()?;

        // Exit the root memory region
        self.memory_analyzer.exit_region()?;

        self.stats.modules_analyzed += 1;

        // Module was already cached before analyzing function bodies (line 205)

        self.current_module = prev_sa_module;
        self.type_checker.borrow_mut().set_current_module(prev_tc_module);

        Ok(())
    }

    /// Analyze an import statement
    fn analyze_import(&mut self, import: &ImportStatement) -> Result<(), SemanticError> {
        let module_name = &import.module_name.name;
        let alias = import.alias.as_ref().map(|a| &a.name);

        // Check if we've already analyzed this module
        if self.analyzed_modules.contains_key(module_name) {
            // Module already loaded and analyzed, just need to add to current scope
            self.add_imported_module_to_scope(module_name, alias, &import.source_location)?;
            return Ok(());
        }

        // Load the module and check for circular dependencies
        let loaded_module = self.module_loader.load_module(module_name).map_err(|e| {
            SemanticError::ImportError {
                module: module_name.clone(),
                reason: format!("Failed to load module: {}", e),
                location: import.source_location.clone(),
            }
        })?;

        // Clone the module and dependencies to avoid borrow issues
        let module_to_analyze = loaded_module.module.clone();
        let loaded_module_clone = loaded_module.clone();

        // Store current module context
        let prev_sa_module = self.current_module.clone();
        let prev_tc_module = self.type_checker.borrow().get_current_module();

        // Analyze the imported module
        self.current_module = Some(module_name.clone());
        self.type_checker.borrow_mut().set_current_module(self.current_module.clone());

        if let Err(e) = self.analyze_module(&module_to_analyze) {
            self.current_module = prev_sa_module;
            self.type_checker.borrow_mut().set_current_module(prev_tc_module.clone());
            return Err(SemanticError::ImportError {
                module: module_name.clone(),
                reason: format!("Failed to analyze module: {}", e),
                location: import.source_location.clone(),
            });
        }

        // Restore module context
        self.current_module = prev_sa_module;
        self.type_checker.borrow_mut().set_current_module(prev_tc_module);

        // Cache the analyzed module
        self.analyzed_modules
            .insert(module_name.clone(), loaded_module_clone);

        // Add imported module to current scope
        self.add_imported_module_to_scope(module_name, alias, &import.source_location)?;

        Ok(())
    }

    /// Add imported module symbols to current scope
    fn add_imported_module_to_scope(
        &mut self,
        module_name: &str,
        alias: Option<&String>,
        location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // Get the loaded module
        let loaded_module =
            self.analyzed_modules
                .get(module_name)
                .ok_or_else(|| SemanticError::Internal {
                    message: format!("Module {} not found in analyzed modules cache", module_name),
                })?;

        let mut exported_symbols = HashMap::new();

        // Add the module itself as a symbol so "io.println" works if "io" is used as prefix
        let module_symbol_name = if let Some(alias_name) = alias {
            alias_name.clone()
        } else {
            module_name.to_string()
        };

        let module_symbol = Symbol::new(
            module_symbol_name.clone(),
            Type::Module(module_name.to_string()),
            SymbolKind::Module,
            false,
            true,
            location.clone(),
        );
        eprintln!("DEBUG: Adding module symbol '{}' with Type::Module('{}') to global scope", module_symbol_name, module_name);
        // Add to global scope so it's accessible during MIR lowering
        self.symbol_table.add_symbol_to_global(module_symbol.clone())?;



        // Process exports from the imported module
        for export in &loaded_module.module.exports {
            match export {
                ExportStatement::Function { name, .. } => {
                    // Add exported function to symbol table with module prefix
                    let qualified_name = if let Some(alias_name) = alias {
                        format!("{}.{}", alias_name, name.name)
                    } else {
                        format!("{}.{}", module_name, name.name)
                    };

                    // Resolve function signature from AST, including FFI symbol for externals
                    let (parameter_types, return_type, is_variadic, ffi_symbol) = {
                        if let Some(func) = loaded_module
                            .module
                            .function_definitions
                            .iter()
                            .find(|f| f.name.name == name.name)
                        {
                            let mut params = Vec::new();
                            for param in &func.parameters {
                                let p_type = self
                                    .type_checker
                                    .borrow()
                                    .ast_type_to_type(&param.param_type)
                                    .unwrap_or(Type::Error);
                                params.push(p_type);
                            }
                            let ret_type = self
                                .type_checker
                                .borrow()
                                .ast_type_to_type(&func.return_type)
                                .unwrap_or(Type::Error);
                            (params, Box::new(ret_type), false, None)
                        } else if let Some(func) = loaded_module
                            .module
                            .external_functions
                            .iter()
                            .find(|f| f.name.name == name.name)
                        {
                            let mut params = Vec::new();
                            for param in &func.parameters {
                                let p_type = self
                                    .type_checker
                                    .borrow()
                                    .ast_type_to_type(&param.param_type)
                                    .unwrap_or(Type::Error);
                                params.push(p_type);
                            }
                            let ret_type = self
                                .type_checker
                                .borrow()
                                .ast_type_to_type(&func.return_type)
                                .unwrap_or(Type::Error);
                            // Capture the FFI symbol from the extern annotation
                            (params, Box::new(ret_type), func.variadic, func.symbol.clone())
                        } else {
                            (
                                vec![],
                                Box::new(Type::Primitive(PrimitiveType::Void)),
                                false,
                                None,
                            )
                        }
                    };

                    let symbol = Symbol::new_with_ffi_symbol(
                        qualified_name.clone(),
                        Type::Function {
                            parameter_types,
                            return_type,
                            is_variadic,
                        },
                        SymbolKind::Function,
                        false,
                        true,
                        location.clone(),
                        ffi_symbol,
                    );
                    // Add to global scope so it's accessible during MIR lowering
                    self.symbol_table.add_symbol_to_global(symbol.clone())?;
                    // Insert with just the function name as key, not qualified name
                    // because lookup_symbol extracts just the method name after the dot
                    exported_symbols.insert(name.name.clone(), symbol.clone());
                }
                ExportStatement::Type { name, .. } => {
                    // Add exported type to type system
                    let qualified_name = if let Some(alias_name) = alias {
                        format!("{}.{}", alias_name, name.name)
                    } else {
                        format!("{}.{}", module_name, name.name)
                    };

                    // Find the type definition in the loaded module
                    if let Some(type_def) =
                        loaded_module
                            .module
                            .type_definitions
                            .iter()
                            .find(|td| match td {
                                crate::ast::TypeDefinition::Structured { name: n, .. } => {
                                    n.name == name.name
                                }
                                crate::ast::TypeDefinition::Enumeration { name: n, .. } => {
                                    n.name == name.name
                                }
                                crate::ast::TypeDefinition::Alias { new_name: n, .. } => {
                                    n.name == name.name
                                }
                            })
                    {
                        // Convert AST definition to TypeDefinition (omitted for brevity, existing logic)
                        // ... (keep existing logic) ...

                        // Refactoring analyze_type_definition is better but risky.
                        // Let's implement minimal conversion matching analyze_type_definition logic.

                        let definition = match type_def {
                            crate::ast::TypeDefinition::Structured {
                                fields,
                                source_location,
                                ..
                            } => {
                                let mut field_types = Vec::new();
                                for field in fields {
                                    // Note: We might fail to resolve types used inside the struct if they are not fully qualified
                                    // This is a known limitation of this simple import mechanism.
                                    // Assuming types are primitive or fully qualified.
                                    let field_type = self
                                        .type_checker
                                        .borrow()
                                        .ast_type_to_type(&field.field_type)
                                        .unwrap_or(Type::Error);
                                    field_types.push((field.name.name.clone(), field_type));
                                }
                                crate::types::TypeDefinition::Struct {
                                    fields: field_types,
                                    source_location: source_location.clone(),
                                }
                            }
                            crate::ast::TypeDefinition::Enumeration {
                                variants,
                                source_location,
                                ..
                            } => {
                                let mut variant_infos = Vec::new();
                                for (idx, variant) in variants.iter().enumerate() {
                                    let mut associated_types = Vec::new();
                                    for type_spec in &variant.associated_types {
                                        let t = self
                                            .type_checker
                                            .borrow()
                                            .ast_type_to_type(type_spec)
                                            .unwrap_or(Type::Error);
                                        associated_types.push(t);
                                    }

                                                                        variant_infos.push(crate::types::EnumVariantInfo {
                                                                            name: variant.name.name.clone(),
                                                                            associated_types,
                                                                            discriminant: idx, // Simple discriminant
                                                                        });                                }
                                crate::types::TypeDefinition::Enum {
                                    variants: variant_infos,
                                    source_location: source_location.clone(),
                                }
                            }
                            crate::ast::TypeDefinition::Alias {
                                original_type,
                                source_location,
                                ..
                            } => {
                                let target = self
                                    .type_checker
                                    .borrow()
                                    .ast_type_to_type(original_type)
                                    .unwrap_or(Type::Error);
                                crate::types::TypeDefinition::Alias {
                                    target_type: target,
                                    source_location: source_location.clone(),
                                }
                            }
                        };

                        // Only add if not already defined
                        if self.symbol_table.lookup_type_definition(&qualified_name).is_none() {
                            self.symbol_table
                                .add_type_definition(qualified_name.clone(), definition.clone())?;
                            self.type_checker
                                .borrow_mut()
                                .add_type_definition(qualified_name.clone(), definition);
                        }
                    } else if let Some(trait_def) = loaded_module
                        .module
                        .trait_definitions
                        .iter()
                        .find(|td| td.name.name == name.name)
                    {
                        // Register imported trait definition if not already present
                        if !self.trait_definitions.contains_key(&qualified_name) {
                            self.trait_definitions
                                .insert(qualified_name.clone(), trait_def.clone());
                        }
                    }

                    let symbol = Symbol::new(
                        qualified_name.clone(),
                        Type::Named {
                            name: name.name.clone(),
                            module: Some(module_name.to_string()),
                        },
                        SymbolKind::Type,
                        false,
                        true,
                        location.clone(),
                    );
                    // Add to global scope so it's accessible during MIR lowering
                    self.symbol_table.add_symbol_to_global(symbol.clone())?;
                    // Insert with just the type name as key
                    exported_symbols.insert(name.name.clone(), symbol.clone());
                }
                ExportStatement::Constant { name, .. } => {
                    // Add exported constant to symbol table
                    let qualified_name = if let Some(alias_name) = alias {
                        format!("{}.{}", alias_name, name.name)
                    } else {
                        format!("{}.{}", module_name, name.name)
                    };

                    // For now, add with Unknown type - full implementation would
                    // need to track constant values and types
                    let symbol = Symbol::new(
                        qualified_name.clone(),
                        Type::Error, // Use Error type as placeholder for unknown constant type
                        SymbolKind::Constant,
                        false,
                        true,
                        location.clone(),
                    );

                    self.symbol_table.add_symbol_to_global(symbol.clone())?;
                    // Insert with just the constant name as key
                    exported_symbols.insert(name.name.clone(), symbol);
                }
            }
        }

        self.symbol_table
            .add_import(module_name.to_string(), exported_symbols);
        Ok(())
    }

    /// Helper function to substitute generic types in a given Type
    fn substitute_type(&self, ty: &Type, type_map: &HashMap<String, Type>) -> Type {
        match ty {
            Type::Generic { name, .. } => {
                if let Some(concrete) = type_map.get(name) {
                    concrete.clone()
                } else {
                    ty.clone()
                }
            }
            Type::Array { element_type, size } => Type::Array {
                element_type: Box::new(self.substitute_type(element_type, type_map)),
                size: *size,
            },
            Type::Map {
                key_type,
                value_type,
            } => Type::Map {
                key_type: Box::new(self.substitute_type(key_type, type_map)),
                value_type: Box::new(self.substitute_type(value_type, type_map)),
            },
            Type::Pointer {
                target_type,
                is_mutable,
            } => Type::Pointer {
                target_type: Box::new(self.substitute_type(target_type, type_map)),
                is_mutable: *is_mutable,
            },
            Type::Owned {
                base_type,
                ownership,
            } => Type::Owned {
                base_type: Box::new(self.substitute_type(base_type, type_map)),
                ownership: *ownership,
            },
            Type::Function {
                parameter_types,
                return_type,
                is_variadic,
            } => {
                let substituted_params = parameter_types
                    .iter()
                    .map(|p_ty| self.substitute_type(p_ty, type_map))
                    .collect();
                let substituted_return = Box::new(self.substitute_type(return_type, type_map));
                Type::Function {
                    parameter_types: substituted_params,
                    return_type: substituted_return,
                    is_variadic: *is_variadic,
                }
            }
            Type::GenericInstance {
                base_type,
                type_arguments,
                module,
            } => {
                let substituted_args = type_arguments
                    .iter()
                    .map(|arg_ty| self.substitute_type(arg_ty, type_map))
                    .collect();
                Type::GenericInstance {
                    base_type: base_type.clone(),
                    type_arguments: substituted_args,
                    module: module.clone(),
                }
            }
            _ => ty.clone(),
        }
    }

    /// Replace occurrences of `Self` in a type with the concrete receiver type
    fn substitute_self_type(&self, ty: &Type, self_type: &Type) -> Type {
        match ty {
            Type::Named { name, module } if name == "Self" => {
                // Preserve module information if present on the receiver
                match self_type {
                    Type::Named { .. } | Type::GenericInstance { .. } | Type::Generic { .. } => {
                        self_type.clone()
                    }
                    _ => Type::Named {
                        name: name.clone(),
                        module: module.clone(),
                    },
                }
            }
            Type::Array { element_type, size } => Type::Array {
                element_type: Box::new(self.substitute_self_type(element_type, self_type)),
                size: *size,
            },
            Type::Map {
                key_type,
                value_type,
            } => Type::Map {
                key_type: Box::new(self.substitute_self_type(key_type, self_type)),
                value_type: Box::new(self.substitute_self_type(value_type, self_type)),
            },
            Type::Pointer {
                target_type,
                is_mutable,
            } => Type::Pointer {
                target_type: Box::new(self.substitute_self_type(target_type, self_type)),
                is_mutable: *is_mutable,
            },
            Type::Owned {
                base_type,
                ownership,
            } => Type::Owned {
                base_type: Box::new(self.substitute_self_type(base_type, self_type)),
                ownership: *ownership,
            },
            Type::Function {
                parameter_types,
                return_type,
                is_variadic,
            } => {
                let params: Vec<Type> = parameter_types
                    .iter()
                    .map(|p| self.substitute_self_type(p, self_type))
                    .collect();
                let ret = Box::new(self.substitute_self_type(return_type, self_type));
                Type::Function {
                    parameter_types: params,
                    return_type: ret,
                    is_variadic: *is_variadic,
                }
            }
            Type::GenericInstance {
                base_type,
                type_arguments,
                module,
            } => Type::GenericInstance {
                base_type: base_type.clone(),
                type_arguments: type_arguments
                    .iter()
                    .map(|arg| self.substitute_self_type(arg, self_type))
                    .collect(),
                module: module.clone(),
            },
            _ => ty.clone(),
        }
    }

    /// Convert AST type to a Type, allowing `Self` as a placeholder before substitution
    fn ast_type_to_type_allow_self(
        &self,
        type_spec: &TypeSpecifier,
    ) -> Result<Type, SemanticError> {
        if let TypeSpecifier::Named { name, .. } = type_spec {
            if name.name == "Self" {
                return Ok(Type::named("Self".to_string(), self.current_module.clone()));
            }
        }
        self.type_checker.borrow().ast_type_to_type(type_spec)
    }

    /// Compute a method signature after substituting generics and `Self`
    fn compute_method_signature(
        &self,
        method_params: &[Parameter],
        method_return: &TypeSpecifier,
        trait_generics: &[GenericParameter],
        method_generics: &[GenericParameter],
        self_type: &Type,
        substitutions: &HashMap<String, Type>,
    ) -> Result<MethodSignatureInfo, SemanticError> {
        {
            let mut checker = self.type_checker.borrow_mut();
            checker.enter_generic_scope();
            for gen in trait_generics {
                checker.add_generic_param(gen.name.name.clone());
            }
            for gen in method_generics {
                checker.add_generic_param(gen.name.name.clone());
            }
        }

        let result: Result<MethodSignatureInfo, SemanticError> = (|| {
            let mut self_param_type = None;
            let mut call_param_types = Vec::new();
            let mut all_param_types = Vec::new();

            for (idx, param) in method_params.iter().enumerate() {
                let raw_type = self.ast_type_to_type_allow_self(&param.param_type)?;
                let with_self = self.substitute_self_type(&raw_type, self_type);
                let substituted = self.substitute_type(&with_self, substitutions);
                all_param_types.push(substituted.clone());

                if idx == 0 && param.name.name == "self" {
                    self_param_type = Some(substituted);
                } else {
                    call_param_types.push(substituted);
                }
            }

            let raw_return = self.ast_type_to_type_allow_self(method_return)?;
            let with_self_return = self.substitute_self_type(&raw_return, self_type);
            let return_type = self.substitute_type(&with_self_return, substitutions);

            Ok(MethodSignatureInfo {
                self_param_type,
                call_param_types,
                all_param_types,
                return_type,
            })
        })();

        {
            let mut checker = self.type_checker.borrow_mut();
            checker.exit_generic_scope();
        }

        result
    }

    /// Register a trait definition for later method resolution
    fn analyze_trait_definition(
        &mut self,
        trait_def: &TraitDefinition,
    ) -> Result<(), SemanticError> {
        // Track generic parameters for trait-level type resolution
        {
            let mut checker = self.type_checker.borrow_mut();
            checker.enter_generic_scope();
            for gen in &trait_def.generic_parameters {
                checker.add_generic_param(gen.name.name.clone());
            }
        }

        self.trait_definitions
            .insert(trait_def.name.name.clone(), trait_def.clone());

        {
            let mut checker = self.type_checker.borrow_mut();
            checker.exit_generic_scope();
        }

        Ok(())
    }

    /// Analyze an impl block (trait or inherent) and register dispatch info
    fn analyze_impl_block(&mut self, impl_block: &TraitImpl) -> Result<(), SemanticError> {
        // Track impl-level generics
        {
            let mut checker = self.type_checker.borrow_mut();
            checker.enter_generic_scope();
            for gen in &impl_block.generic_parameters {
                checker.add_generic_param(gen.name.name.clone());
            }
        }

        let result: Result<(), SemanticError> = (|| {
            // Resolve the concrete type being implemented
            let self_type = self
                .type_checker
                .borrow()
                .ast_type_to_type(&impl_block.for_type)?;

            // Build substitutions for trait generic parameters, if any
            let mut trait_substitutions: HashMap<String, Type> = HashMap::new();
            if let Some(trait_ident) = &impl_block.trait_name {
                if let Some(trait_def) = self.trait_definitions.get(&trait_ident.name) {
                    if trait_def.generic_parameters.len() != impl_block.trait_generic_args.len() {
                        return Err(SemanticError::GenericArgumentCountMismatch {
                            function: trait_ident.name.clone(),
                            expected: trait_def.generic_parameters.len(),
                            found: impl_block.trait_generic_args.len(),
                            location: impl_block.source_location.clone(),
                        });
                    }

                    for (gen, arg_spec) in trait_def
                        .generic_parameters
                        .iter()
                        .zip(impl_block.trait_generic_args.iter())
                    {
                        let arg_type = self.type_checker.borrow().ast_type_to_type(arg_spec)?;
                        trait_substitutions.insert(gen.name.name.clone(), arg_type);
                    }
                } else {
                    return Err(SemanticError::UndefinedSymbol {
                        symbol: trait_ident.name.clone(),
                        location: impl_block.source_location.clone(),
                    });
                }
            }

            // First pass: register signatures and dispatch entries
            let mut impl_signatures: HashMap<String, MethodSignatureInfo> = HashMap::new();
            for method in &impl_block.methods {
                let trait_generics = if let Some(trait_ident) = &impl_block.trait_name {
                    self.trait_definitions
                        .get(&trait_ident.name)
                        .map(|t| t.generic_parameters.clone())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                let signature = self.compute_method_signature(
                    &method.parameters,
                    &method.return_type,
                    &trait_generics,
                    &method.generic_parameters,
                    &self_type,
                    &trait_substitutions,
                )?;

                let symbol_name = if let Some(trait_ident) = &impl_block.trait_name {
                    format!("{}::{}", trait_ident.name, method.name.name)
                } else {
                    format!("{}::{}", self_type, method.name.name)
                };

                let func_type = Type::function(
                    signature.all_param_types.clone(),
                    signature.return_type.clone(),
                );

                impl_signatures.insert(method.name.name.clone(), signature.clone());

                let func_symbol = Symbol {
                    name: symbol_name.clone(),
                    symbol_type: func_type,
                    kind: SymbolKind::Function,
                    is_mutable: false,
                    is_initialized: true,
                    declaration_location: method.source_location.clone(),
                    is_moved: false,
                    borrow_state: BorrowState::None,
                    ffi_symbol: None,
                };
                // Signature-only registration; errors about duplicates bubble up
                self.symbol_table.add_symbol(func_symbol)?;

                if let Some(trait_ident) = &impl_block.trait_name {
                    let key = TraitMethodKey {
                        receiver: self_type.clone(),
                        method_name: method.name.name.clone(),
                    };
                    let dispatch = TraitMethodDispatch {
                        trait_name: trait_ident.name.clone(),
                        impl_type: self_type.clone(),
                        method_name: method.name.name.clone(),
                        self_param_type: signature.self_param_type.clone(),
                        param_types: signature.call_param_types.clone(),
                        return_type: signature.return_type.clone(),
                        symbol_name,
                    };
                    self.trait_dispatch_table.insert(key, dispatch);
                }
            }

            // Verify required trait methods are implemented and signatures match
            if let Some(trait_ident) = &impl_block.trait_name {
                if let Some(trait_def) = self.trait_definitions.get(&trait_ident.name) {
                    for trait_method in &trait_def.methods {
                        if trait_method.default_body.is_some() {
                            continue;
                        }

                        let expected_sig = self.compute_method_signature(
                            &trait_method.parameters,
                            &trait_method.return_type,
                            &trait_def.generic_parameters,
                            &trait_method.generic_parameters,
                            &self_type,
                            &trait_substitutions,
                        )?;

                        if let Some(impl_sig) = impl_signatures.get(&trait_method.name.name) {
                            if expected_sig.call_param_types.len()
                                != impl_sig.call_param_types.len()
                            {
                                return Err(SemanticError::TraitMethodSignatureMismatch {
                                    trait_name: trait_ident.name.clone(),
                                    method_name: trait_method.name.name.clone(),
                                    impl_type: self_type.to_string(),
                                    expected: format!(
                                        "fn({}) -> {}",
                                        expected_sig
                                            .call_param_types
                                            .iter()
                                            .map(|t| t.to_string())
                                            .collect::<Vec<_>>()
                                            .join(", "),
                                        expected_sig.return_type
                                    ),
                                    found: format!(
                                        "fn({}) -> {}",
                                        impl_sig
                                            .call_param_types
                                            .iter()
                                            .map(|t| t.to_string())
                                            .collect::<Vec<_>>()
                                            .join(", "),
                                        impl_sig.return_type
                                    ),
                                    location: impl_block.source_location.clone(),
                                });
                            }

                            for (expected_ty, actual_ty) in expected_sig
                                .call_param_types
                                .iter()
                                .zip(impl_sig.call_param_types.iter())
                            {
                                if !self
                                    .type_checker
                                    .borrow()
                                    .types_compatible(expected_ty, actual_ty)
                                {
                                    return Err(SemanticError::TraitMethodSignatureMismatch {
                                        trait_name: trait_ident.name.clone(),
                                        method_name: trait_method.name.name.clone(),
                                        impl_type: self_type.to_string(),
                                        expected: expected_ty.to_string(),
                                        found: actual_ty.to_string(),
                                        location: trait_method.source_location.clone(),
                                    });
                                }
                            }

                            if !self
                                .type_checker
                                .borrow()
                                .types_compatible(&expected_sig.return_type, &impl_sig.return_type)
                            {
                                return Err(SemanticError::TraitMethodSignatureMismatch {
                                    trait_name: trait_ident.name.clone(),
                                    method_name: trait_method.name.name.clone(),
                                    impl_type: self_type.to_string(),
                                    expected: expected_sig.return_type.to_string(),
                                    found: impl_sig.return_type.to_string(),
                                    location: trait_method.source_location.clone(),
                                });
                            }
                        } else {
                            return Err(SemanticError::TraitMethodNotImplemented {
                                trait_name: trait_ident.name.clone(),
                                method_name: trait_method.name.name.clone(),
                                impl_type: self_type.to_string(),
                                location: impl_block.source_location.clone(),
                            });
                        }
                    }
                }
            }

            // Second pass: analyze method bodies with namespaced symbols
            for method in &impl_block.methods {
                let mut namespaced = method.clone();
                if let Some(trait_ident) = &impl_block.trait_name {
                    namespaced.name.name = format!("{}::{}", trait_ident.name, method.name.name);
                } else {
                    namespaced.name.name = format!("{}::{}", self_type, method.name.name);
                }
                self.analyze_function_body(&namespaced)?;
            }

            Ok(())
        })();

        {
            let mut checker = self.type_checker.borrow_mut();
            checker.exit_generic_scope();
        }

        result
    }

    /// Analyze a type definition
    fn analyze_type_definition(
        &mut self,
        type_def: &crate::ast::TypeDefinition,
    ) -> Result<(), SemanticError> {
        // Enter a new scope for the type definition to hold generic parameters
        self.symbol_table.enter_scope(ScopeKind::Block); // Use Block scope for now, could be a dedicated TypeScope

        match type_def {
            crate::ast::TypeDefinition::Structured {
                name,
                generic_parameters,
                fields,
                source_location,
                ..
            } => {
                // Enter generic scope on type checker FIRST
                self.type_checker.borrow_mut().enter_generic_scope();
                for generic_param in generic_parameters {
                    self.type_checker
                        .borrow_mut()
                        .add_generic_param(generic_param.name.name.clone());
                }

                // Add generic parameters to symbol table as well
                for generic_param in generic_parameters {
                    let generic_type = self.type_checker.borrow().ast_type_to_type(
                        &TypeSpecifier::TypeParameter {
                            name: generic_param.name.clone(),
                            constraints: generic_param.constraints.clone(),
                            source_location: generic_param.source_location.clone(),
                        },
                    )?;
                    let generic_symbol = Symbol::new(
                        generic_param.name.name.clone(),
                        generic_type,
                        SymbolKind::Type,
                        false,
                        true,
                        generic_param.source_location.clone(),
                    );
                    self.symbol_table.add_symbol(generic_symbol)?;
                }

                let mut field_types = Vec::new();

                // Analyze each field (preserving declaration order)
                for field in fields {
                    let field_type = self
                        .type_checker
                        .borrow()
                        .ast_type_to_type(&field.field_type)?;
                    field_types.push((field.name.name.clone(), field_type));
                }

                // Exit generic scope on type checker
                self.type_checker.borrow_mut().exit_generic_scope();

                // Add the type definition
                let definition = crate::types::TypeDefinition::Struct {
                    fields: field_types.clone(),
                    source_location: source_location.clone(),
                };

                self.symbol_table
                    .add_type_definition(name.name.clone(), definition.clone())?;
                self.type_checker
                    .borrow_mut()
                    .add_type_definition(name.name.clone(), definition);
            }

            crate::ast::TypeDefinition::Enumeration {
                name,
                generic_parameters,
                variants,
                source_location,
                ..
            } => {
                // Enter generic scope on type checker FIRST
                self.type_checker.borrow_mut().enter_generic_scope();
                for generic_param in generic_parameters {
                    self.type_checker
                        .borrow_mut()
                        .add_generic_param(generic_param.name.name.clone());
                }

                // Add generic parameters to symbol table as well
                for generic_param in generic_parameters {
                    let generic_type = self.type_checker.borrow().ast_type_to_type(
                        &TypeSpecifier::TypeParameter {
                            name: generic_param.name.clone(),
                            constraints: generic_param.constraints.clone(),
                            source_location: generic_param.source_location.clone(),
                        },
                    )?;
                    let generic_symbol = Symbol::new(
                        generic_param.name.name.clone(),
                        generic_type,
                        SymbolKind::Type,
                        false,
                        true,
                        generic_param.source_location.clone(),
                    );
                    self.symbol_table.add_symbol(generic_symbol)?;
                }

                // Convert AST variants to type system variants
                let mut variant_infos = Vec::new();
                for (idx, variant) in variants.iter().enumerate() {
                    let mut associated_types = Vec::new();
                    for type_spec in &variant.associated_types {
                        associated_types
                            .push(self.type_checker.borrow().ast_type_to_type(type_spec)?);
                    }

                    variant_infos.push(crate::types::EnumVariantInfo {
                        name: variant.name.name.clone(),
                        associated_types,
                        discriminant: idx, // Variants get indices based on declaration order
                    });
                }

                // Exit generic scope on type checker
                self.type_checker.borrow_mut().exit_generic_scope();

                let definition = crate::types::TypeDefinition::Enum {
                    variants: variant_infos.clone(),
                    source_location: source_location.clone(),
                };

                self.symbol_table
                    .add_type_definition(name.name.clone(), definition.clone())?;
                self.type_checker
                    .borrow_mut()
                    .add_type_definition(name.name.clone(), definition);
            }

            crate::ast::TypeDefinition::Alias {
                new_name,
                original_type,
                source_location,
                ..
            } => {
                let target_type = self.type_checker.borrow().ast_type_to_type(original_type)?;

                let definition = crate::types::TypeDefinition::Alias {
                    target_type,
                    source_location: source_location.clone(),
                };

                self.symbol_table
                    .add_type_definition(new_name.name.clone(), definition.clone())?;
                self.type_checker
                    .borrow_mut()
                    .add_type_definition(new_name.name.clone(), definition);
            }
        }

        self.stats.types_defined += 1;
        self.symbol_table.exit_scope()?; // Exit the temporary scope for generics
        Ok(())
    }

    /// Analyze a constant declaration
    fn analyze_constant_declaration(
        &mut self,
        const_decl: &ConstantDeclaration,
    ) -> Result<(), SemanticError> {
        // Get the declared type
        let declared_type = self
            .type_checker
            .borrow()
            .ast_type_to_type(&const_decl.type_spec)?;

        // Analyze the value expression
        let value_type = self.analyze_expression(&const_decl.value)?;

        // Check type compatibility
        if !self
            .type_checker
            .borrow()
            .types_compatible(&declared_type, &value_type)
        {
            return Err(SemanticError::TypeMismatch {
                expected: declared_type.to_string(),
                found: value_type.to_string(),
                location: const_decl.source_location.clone(),
            });
        }

        // Add the constant to the symbol table
        let symbol = Symbol {
            name: const_decl.name.name.clone(),
            symbol_type: declared_type,
            kind: SymbolKind::Constant,
            is_mutable: false,
            is_initialized: true,
            declaration_location: const_decl.source_location.clone(),
            is_moved: false,
            borrow_state: BorrowState::None,
            ffi_symbol: None,
        };

        self.symbol_table.add_symbol(symbol)?;
        self.stats.variables_declared += 1;

        Ok(())
    }

    /// Add function signature to symbol table (first pass)
    fn add_function_signature(&mut self, func_def: &Function) -> Result<(), SemanticError> {
        // Enter generic scope on type checker FIRST
        // This must happen before analyzing parameter types and return type
        self.type_checker.borrow_mut().enter_generic_scope();
        for generic_param in &func_def.generic_parameters {
            self.type_checker
                .borrow_mut()
                .add_generic_param(generic_param.name.name.clone());
        }

        // Get the return type (now with generic params in scope)
        let return_type = self
            .type_checker
            .borrow()
            .ast_type_to_type(&func_def.return_type)?;

        // Analyze parameters
        let mut param_types = Vec::new();
        for param in &func_def.parameters {
            let param_type = self
                .type_checker
                .borrow()
                .ast_type_to_type(&param.param_type)?;
            param_types.push(param_type);
        }

        // Exit generic scope on type checker
        self.type_checker.borrow_mut().exit_generic_scope();

        // Create function type
        let func_type = Type::function(param_types, return_type);

        // Add function to symbol table
        let func_symbol = Symbol {
            name: func_def.name.name.clone(),
            symbol_type: func_type,
            kind: SymbolKind::Function,
            is_mutable: false,
            is_initialized: true,
            declaration_location: func_def.source_location.clone(),
            is_moved: false,
            borrow_state: BorrowState::None,
            ffi_symbol: None,
        };

        self.symbol_table.add_symbol(func_symbol)?;
        Ok(())
    }

    /// Analyze function body (second pass)
    fn analyze_function_body(&mut self, func_def: &Function) -> Result<(), SemanticError> {
        // Enter function scope
        self.symbol_table.enter_scope(ScopeKind::Function);

        // Enter generic scope on type checker and add generic parameters FIRST
        // This must happen before analyzing parameter types and return type
        self.type_checker.borrow_mut().enter_generic_scope();
        for generic_param in &func_def.generic_parameters {
            self.type_checker
                .borrow_mut()
                .add_generic_param(generic_param.name.name.clone());
        }

        // Capture where-clause constraints for generic parameters
        self.current_generic_constraints.clear();
        for where_clause in &func_def.where_clause {
            let mut constraints = Vec::new();
            for constraint in &where_clause.constraints {
                constraints.push(crate::types::TypeConstraintInfo::TraitBound {
                    trait_name: constraint.name.clone(),
                    module: self.current_module.clone(),
                });
            }
            self.current_generic_constraints
                .insert(where_clause.type_param.name.clone(), constraints);
        }

        // Now we can safely analyze the return type (generic params are in scope)
        let return_type = self
            .type_checker
            .borrow()
            .ast_type_to_type(&func_def.return_type)?;
        self.current_function_return_type = Some(return_type);

        // Add generic parameters to symbol table as well
        for generic_param in &func_def.generic_parameters {
            let generic_type =
                self.type_checker
                    .borrow()
                    .ast_type_to_type(&TypeSpecifier::TypeParameter {
                        name: generic_param.name.clone(),
                        constraints: generic_param.constraints.clone(),
                        source_location: generic_param.source_location.clone(),
                    })?;
            let generic_symbol = Symbol::new(
                generic_param.name.name.clone(),
                generic_type,
                SymbolKind::Type,
                false,
                true,
                generic_param.source_location.clone(),
            );
            self.symbol_table.add_symbol(generic_symbol)?;
        }

        // Add parameters to function scope
        for param in &func_def.parameters {
            let param_type = self
                .type_checker
                .borrow()
                .ast_type_to_type(&param.param_type)?;
            let param_symbol = Symbol {
                name: param.name.name.clone(),
                symbol_type: param_type,
                kind: SymbolKind::Parameter,
                is_mutable: true, // Parameters are typically mutable in their scope
                is_initialized: true,
                declaration_location: param.source_location.clone(),
                is_moved: false,
                borrow_state: BorrowState::None,
                ffi_symbol: None,
            };

            self.symbol_table.add_symbol(param_symbol)?;
        }

        // Analyze memory allocation strategy for this function
        let memory_info = self.memory_analyzer.analyze_function(func_def)?;

        // TODO: Store memory_info for later use in code generation
        // For now, we'll just log it in debug mode
        #[cfg(debug_assertions)]
        {
            eprintln!(
                "Memory analysis for function '{}': {:?}",
                func_def.name.name, memory_info
            );
        }

        // Analyze function body
        self.analyze_block(&func_def.body)?;

        // Validate function metadata and contracts
        self.validate_function_contracts(func_def)?;

        // Exit generic scope on type checker
        self.type_checker.borrow_mut().exit_generic_scope();
        self.current_generic_constraints.clear();

        // Exit function scope
        self.symbol_table.exit_scope()?;
        self.stats.functions_analyzed += 1;

        // Clear current function return type
        self.current_function_return_type = None;

        Ok(())
    }

    /// Validate function contracts and metadata
    fn validate_function_contracts(&mut self, func_def: &Function) -> Result<(), SemanticError> {
        // Create contract context
        let mut parameter_types = HashMap::new();
        for param in &func_def.parameters {
            let param_type = self
                .type_checker
                .borrow()
                .ast_type_to_type(&param.param_type)?;
            parameter_types.insert(param.name.name.clone(), param_type);
        }

        let return_type = self
            .type_checker
            .borrow()
            .ast_type_to_type(&func_def.return_type)?;

        let context = ContractContext {
            parameter_types,
            return_type,
            type_checker: self.type_checker.clone(), // Note: This might need a better approach
        };

        // Validate the metadata
        match self.contract_validator.validate_function_metadata(
            &func_def.metadata,
            &context,
            &func_def.name.name,
            &func_def.source_location,
        ) {
            Ok(result) => {
                // Log warnings (in a real implementation, you'd want proper logging)
                for warning in result.warnings {
                    eprintln!(
                        "Contract warning in function '{}': {}",
                        func_def.name.name, warning
                    );
                }

                if !result.is_valid {
                    // Collect all contract errors
                    for error in result.errors {
                        self.errors.push(error);
                    }
                    return Err(SemanticError::InvalidContract {
                        contract_type: "FunctionMetadata".to_string(),
                        reason: "Contract validation failed".to_string(),
                        location: func_def.source_location.clone(),
                    });
                }

                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Analyze an export statement
    fn analyze_export(&mut self, export: &ExportStatement) -> Result<(), SemanticError> {
        match export {
            ExportStatement::Function {
                name,
                source_location,
            }
            | ExportStatement::Constant {
                name,
                source_location,
            } => {
                // Check that the exported symbol exists
                if self.symbol_table.lookup_symbol(&name.name).is_none() {
                    return Err(SemanticError::UndefinedSymbol {
                        symbol: name.name.clone(),
                        location: source_location.clone(),
                    });
                }
            }
            ExportStatement::Type {
                name,
                source_location,
            } => {
                // Check that the exported type exists
                if self
                    .symbol_table
                    .lookup_type_definition(&name.name)
                    .is_none()
                    && !self.trait_definitions.contains_key(&name.name)
                {
                    return Err(SemanticError::UndefinedSymbol {
                        symbol: name.name.clone(),
                        location: source_location.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Helper to look up an AST Function by its qualified or unqualified name
    fn lookup_ast_function_in_modules(&self, name: &str) -> Option<Function> {
        for loaded_module in self.analyzed_modules.values() {
            for func_def in &loaded_module.module.function_definitions {
                // Match by fully qualified name
                let full_name = if loaded_module.module.name.name == "main" {
                    func_def.name.name.clone()
                } else {
                    format!("{}.{}", loaded_module.module.name.name, func_def.name.name)
                };
                if full_name == name {
                    return Some(func_def.clone());
                }
                // Also match by simple name (for calls within the same module)
                if func_def.name.name == name {
                    return Some(func_def.clone());
                }
            }
        }
        None
    }

    /// Analyze a block of statements
    fn analyze_block(&mut self, block: &Block) -> Result<(), SemanticError> {
        self.symbol_table.enter_scope(ScopeKind::Block);

        for statement in &block.statements {
            self.analyze_statement(statement)?;
        }

        self.symbol_table.exit_scope()?;
        Ok(())
    }

    /// Analyze a lambda block and return the inferred return type
    /// This differs from analyze_block in that it tracks return types
    fn analyze_lambda_block_return_type(&mut self, block: &Block) -> Result<Type, SemanticError> {
        // Don't create a new scope - lambda already created one
        let mut return_type: Option<Type> = None;

        for statement in &block.statements {
            // Check for return statements and track their types
            if let Statement::Return {
                value,
                source_location,
            } = statement
            {
                let stmt_return_type = if let Some(expr) = value {
                    self.analyze_expression(expr)?
                } else {
                    Type::primitive(PrimitiveType::Void)
                };

                if let Some(ref existing_type) = return_type {
                    // Check consistency of return types
                    if !self
                        .type_checker
                        .borrow()
                        .are_types_equal(existing_type, &stmt_return_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: existing_type.to_string(),
                            found: stmt_return_type.to_string(),
                            location: source_location.clone(),
                        });
                    }
                } else {
                    return_type = Some(stmt_return_type);
                }
            } else {
                // Analyze other statements normally
                self.analyze_statement(statement)?;
            }
        }

        // Return the inferred type, or Void if no return statements
        Ok(return_type.unwrap_or_else(|| Type::primitive(PrimitiveType::Void)))
    }

    /// Analyze a statement
    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), SemanticError> {
        match statement {
            Statement::VariableDeclaration {
                name,
                type_spec,
                mutability,
                initial_value,
                source_location,
                ..
            } => {
                let declared_type = self.type_checker.borrow().ast_type_to_type(type_spec)?;
                let is_mutable = matches!(mutability, Mutability::Mutable);
                let mut is_initialized = false;
                let mut final_type = declared_type.clone();

                // If there's an initial value, analyze it and check type compatibility
                if let Some(init_expr) = initial_value {
                    let init_type = self.analyze_expression(init_expr)?;

                    if matches!(declared_type, Type::Variable(_)) {
                        final_type = init_type.clone();
                    } else if !self
                        .type_checker
                        .borrow()
                        .types_compatible(&declared_type, &init_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: declared_type.to_string(),
                            found: init_type.to_string(),
                            location: source_location.clone(),
                        });
                    }

                    // Check ownership transfer (move/borrow)
                    self.check_argument_ownership(init_expr, &final_type)?;

                    is_initialized = true;
                }

                // Add variable to symbol table with the final type
                let symbol = Symbol::new(
                    name.name.clone(),
                    final_type,
                    SymbolKind::Variable,
                    is_mutable,
                    is_initialized,
                    source_location.clone(),
                );

                self.symbol_table.add_symbol(symbol)?;
                self.stats.variables_declared += 1;
            }

            Statement::Assignment {
                target,
                value,
                source_location,
            } => {
                let value_type = self.analyze_expression(value)?;

                match target {
                    AssignmentTarget::Variable { name } => {
                        // Check that variable exists and is mutable
                        // Clone symbol info to avoid holding borrow on symbol_table while calling check_argument_ownership
                        let (symbol_type, is_mutable) = {
                            let symbol =
                                self.symbol_table.lookup_symbol(&name.name).ok_or_else(|| {
                                    SemanticError::UndefinedSymbol {
                                        symbol: name.name.clone(),
                                        location: source_location.clone(),
                                    }
                                })?;
                            (symbol.symbol_type.clone(), symbol.is_mutable)
                        };

                        if !is_mutable {
                            return Err(SemanticError::AssignToImmutable {
                                variable: name.name.clone(),
                                location: source_location.clone(),
                            });
                        }

                        // Check type compatibility
                        if !self
                            .type_checker
                            .borrow()
                            .types_compatible(&symbol_type, &value_type)
                        {
                            return Err(SemanticError::TypeMismatch {
                                expected: symbol_type.to_string(),
                                found: value_type.to_string(),
                                location: source_location.clone(),
                            });
                        }

                        // Check ownership transfer (move/borrow)
                        self.check_argument_ownership(value, &symbol_type)?;

                        // Mark variable as initialized
                        self.symbol_table.mark_variable_initialized(&name.name)?;
                    }

                    // TODO: Handle other assignment targets (array elements, struct fields, etc.)
                    _ => {
                        // For now, just analyze the target as an expression to check types
                        self.analyze_assignment_target(target)?;
                    }
                }
            }

            Statement::Return {
                value,
                source_location,
            } => {
                let return_type = if let Some(return_expr) = value {
                    self.analyze_expression(return_expr)?
                } else {
                    Type::primitive(PrimitiveType::Void)
                };

                if let Some(expected_type) = &self.current_function_return_type {
                    if !self
                        .type_checker
                        .borrow()
                        .types_compatible(expected_type, &return_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.to_string(),
                            found: return_type.to_string(),
                            location: source_location.clone(),
                        });
                    }
                }
            }

            Statement::FunctionCall { call, .. } => {
                // Track borrowed variables
                let mut borrowed_vars = Vec::new();

                // Analyze arguments to track borrows
                for arg in &call.arguments {
                    if let Expression::Variable { name, .. } = arg.value.as_ref() {
                        // Check if this variable is being borrowed
                        if let Some(symbol) = self.symbol_table.lookup_symbol(&name.name) {
                            if matches!(
                                symbol.borrow_state,
                                BorrowState::Borrowed(_) | BorrowState::BorrowedMut
                            ) {
                                borrowed_vars.push(name.name.clone());
                            }
                        }
                    }
                }

                // Analyze the function call
                self.analyze_function_call(call)?;

                // Release borrows after the function call
                for var_name in borrowed_vars {
                    self.symbol_table.release_borrow(&var_name)?;
                }
            }

            Statement::If {
                condition,
                then_block,
                else_ifs,
                else_block,
                ..
            } => {
                self.analyze_if_statement(condition, then_block, else_ifs, else_block)?;
            }

            Statement::WhileLoop {
                condition,
                body,
                invariant,
                ..
            } => {
                self.analyze_while_loop(condition, body, invariant)?;
            }

            Statement::ForEachLoop {
                collection,
                element_binding,
                element_type,
                body,
                ..
            } => {
                self.analyze_for_each_loop(collection, element_binding, element_type, body)?;
            }

            Statement::FixedIterationLoop {
                counter,
                from_value,
                to_value,
                step_value,
                body,
                ..
            } => {
                self.analyze_fixed_iteration_loop(counter, from_value, to_value, step_value, body)?;
            }

            Statement::Break {
                target_label,
                source_location,
            } => {
                self.analyze_break_statement(target_label, source_location)?;
            }

            Statement::Continue {
                target_label,
                source_location,
            } => {
                self.analyze_continue_statement(target_label, source_location)?;
            }

            Statement::TryBlock {
                protected_block,
                catch_clauses,
                finally_block,
                ..
            } => {
                self.analyze_try_block(protected_block, catch_clauses, finally_block)?;
            }

            Statement::Throw {
                exception,
                source_location,
            } => {
                self.analyze_throw_statement(exception, source_location)?;
            }

            Statement::ResourceScope { scope, .. } => {
                self.analyze_resource_scope(scope)?;
            }

            Statement::Expression { expr, .. } => {
                // For expression statements, just analyze the expression
                self.analyze_expression(expr)?;
            }

            Statement::Match { value, arms, .. } => {
                // Analyze the matched value
                let value_type = self.analyze_expression(value)?;

                // Analyze each arm
                for arm in arms {
                    // Enter scope for arm body (and guard) BEFORE analyzing pattern
                    // (because analyze_pattern adds pattern bindings to the current scope)
                    self.symbol_table.enter_scope(ScopeKind::Block);

                    // Analyze the pattern (adds bindings to scope)
                    self.analyze_pattern(&arm.pattern, &value_type)?;

                    // Analyze guard expression if present
                    if let Some(guard) = &arm.guard {
                        let _guard_type = self.analyze_expression(guard)?;
                    }

                    // Analyze the body block
                    for stmt in &arm.body.statements {
                        self.analyze_statement(stmt)?;
                    }
                    let _ = self.symbol_table.exit_scope();
                }
            }

            Statement::Concurrent { block, .. } => {
                // Save previous state
                let prev_concurrent = self.in_concurrent_block;

                // Enter concurrent context
                self.in_concurrent_block = true;

                // Analyze the block
                self.analyze_block(block)?;

                // Restore state
                self.in_concurrent_block = prev_concurrent;
            }
        }

        Ok(())
    }

    /// Analyze an expression and return its type
    fn analyze_expression(&mut self, expression: &Expression) -> Result<Type, SemanticError> {
        match expression {
            Expression::IntegerLiteral { .. } => Ok(Type::primitive(PrimitiveType::Integer)),

            Expression::FloatLiteral { .. } => Ok(Type::primitive(PrimitiveType::Float)),

            Expression::StringLiteral { .. } => Ok(Type::primitive(PrimitiveType::String)),

            Expression::CharacterLiteral { .. } => Ok(Type::primitive(PrimitiveType::Char)),

            Expression::BooleanLiteral { .. } => Ok(Type::primitive(PrimitiveType::Boolean)),

            Expression::NullLiteral { .. } => {
                // Null can be any pointer type - return a generic pointer for now
                Ok(Type::pointer(Type::primitive(PrimitiveType::Void), false))
            }

            Expression::Variable {
                name,
                source_location,
            } => {
                let symbol = self
                    .symbol_table
                    .lookup_symbol(&name.name)
                    .or_else(|| {
                        // Try looking up with module qualification
                        if let Some(module) = &self.current_module {
                            let qualified = format!("{}.{}", module, name.name);
                            self.symbol_table.lookup_symbol(&qualified)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| SemanticError::UndefinedSymbol {
                        symbol: name.name.clone(),
                        location: source_location.clone(),
                    })?;

                // Check if variable is initialized
                if !symbol.is_initialized {
                    return Err(SemanticError::UseBeforeInitialization {
                        variable: name.name.clone(),
                        location: source_location.clone(),
                    });
                }

                // Check if variable has been moved
                if symbol.is_moved {
                    return Err(SemanticError::UseAfterMove {
                        variable: name.name.clone(),
                        location: source_location.clone(),
                    });
                }

                Ok(symbol.symbol_type.clone())
            }

            Expression::Add {
                left,
                right,
                source_location,
            }
            | Expression::Subtract {
                left,
                right,
                source_location,
            }
            | Expression::Multiply {
                left,
                right,
                source_location,
            }
            | Expression::Divide {
                left,
                right,
                source_location,
            }
            | Expression::IntegerDivide {
                left,
                right,
                source_location,
            }
            | Expression::Modulo {
                left,
                right,
                source_location,
            } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                // Both operands must be numeric
                if !left_type.is_numeric() || !right_type.is_numeric() {
                    return Err(SemanticError::TypeMismatch {
                        expected: "numeric type".to_string(),
                        found: format!("{} and {}", left_type, right_type),
                        location: source_location.clone(),
                    });
                }

                // Return the "larger" numeric type
                if left_type.is_float() || right_type.is_float() {
                    Ok(Type::primitive(PrimitiveType::Float))
                } else {
                    Ok(Type::primitive(PrimitiveType::Integer))
                }
            }

            Expression::FunctionCall {
                call,
                source_location,
            } => self.analyze_function_call_expression(call, source_location),

            Expression::StringConcat {
                operands,
                source_location,
            } => {
                // All operands must be strings
                for operand in operands {
                    let operand_type = self.analyze_expression(operand)?;
                    if !matches!(operand_type, Type::Primitive(PrimitiveType::String)) {
                        return Err(SemanticError::TypeMismatch {
                            expected: "String".to_string(),
                            found: operand_type.to_string(),
                            location: source_location.clone(),
                        });
                    }
                }
                Ok(Type::primitive(PrimitiveType::String))
            }

            Expression::StringLength {
                string,
                source_location,
            } => {
                let string_type = self.analyze_expression(string)?;
                if !matches!(string_type, Type::Primitive(PrimitiveType::String)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "String".to_string(),
                        found: string_type.to_string(),
                        location: source_location.clone(),
                    });
                }
                Ok(Type::primitive(PrimitiveType::Integer))
            }
            Expression::StringCharAt {
                string,
                index,
                source_location,
            } => {
                let string_type = self.analyze_expression(string)?;
                let index_type = self.analyze_expression(index)?;

                if !matches!(string_type, Type::Primitive(PrimitiveType::String)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "String".to_string(),
                        found: string_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                if !matches!(index_type, Type::Primitive(PrimitiveType::Integer)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Integer".to_string(),
                        found: index_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                Ok(Type::primitive(PrimitiveType::Char))
            }

            Expression::Substring {
                string,
                start_index,
                length,
                source_location,
            } => {
                // String argument must be string type
                let string_type = self.analyze_expression(string)?;
                if !matches!(string_type, Type::Primitive(PrimitiveType::String)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "String".to_string(),
                        found: string_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                // Start index must be integer
                let start_type = self.analyze_expression(start_index)?;
                if !matches!(start_type, Type::Primitive(PrimitiveType::Integer)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Integer".to_string(),
                        found: start_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                // Length must be integer
                let length_type = self.analyze_expression(length)?;
                if !matches!(length_type, Type::Primitive(PrimitiveType::Integer)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Integer".to_string(),
                        found: length_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                Ok(Type::primitive(PrimitiveType::String))
            }

            Expression::StringEquals {
                left,
                right,
                source_location,
            } => {
                // Both operands must be strings
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                if !matches!(left_type, Type::Primitive(PrimitiveType::String)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "String".to_string(),
                        found: left_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                if !matches!(right_type, Type::Primitive(PrimitiveType::String)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "String".to_string(),
                        found: right_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                Ok(Type::primitive(PrimitiveType::Boolean))
            }

            Expression::StringContains {
                haystack,
                needle,
                source_location,
            } => {
                // Both operands must be strings
                let haystack_type = self.analyze_expression(haystack)?;
                let needle_type = self.analyze_expression(needle)?;

                if !matches!(haystack_type, Type::Primitive(PrimitiveType::String)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "String".to_string(),
                        found: haystack_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                if !matches!(needle_type, Type::Primitive(PrimitiveType::String)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "String".to_string(),
                        found: needle_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                Ok(Type::primitive(PrimitiveType::Boolean))
            }

            Expression::ArrayLiteral {
                element_type,
                elements,
                source_location,
            } => {
                // Convert AST type to semantic type
                let expected_element_type =
                    self.type_checker.borrow().ast_type_to_type(element_type)?;

                // Check all elements match the declared type
                for element in elements {
                    let element_type = self.analyze_expression(element)?;
                    if !self
                        .type_checker
                        .borrow()
                        .types_compatible(&expected_element_type, &element_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_element_type.to_string(),
                            found: element_type.to_string(),
                            location: source_location.clone(),
                        });
                    }
                }

                Ok(Type::array(expected_element_type, Some(elements.len())))
            }

            Expression::ArrayAccess {
                array,
                index,
                source_location: _,
            } => {
                let array_type = self.analyze_expression(array)?;

                // Check that it's an array
                match array_type {
                                        Type::Array { ref element_type, .. } => {
                                            // Index must be integer
                                            let index_type = self.analyze_expression(index)?;
                                            if !matches!(index_type, Type::Primitive(PrimitiveType::Integer)) {
                                                return Err(SemanticError::TypeMismatch {
                                                    expected: "Integer".to_string(),
                                                    found: index_type.to_string(),
                                                    location: SourceLocation::unknown(),
                                                });
                                            }
                                            Ok(*(*element_type).clone())
                                        }
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "Array".to_string(),
                        found: array_type.to_string(),
                        location: SourceLocation::unknown(),
                    }),
                }
            }

            Expression::ArrayLength {
                array,
                source_location,
            } => {
                let array_type = self.analyze_expression(array)?;

                // Check that it's an array
                match array_type {
                    Type::Array { .. } => Ok(Type::primitive(PrimitiveType::Integer)),
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "Array".to_string(),
                        found: array_type.to_string(),
                        location: source_location.clone(),
                    }),
                }
            }

            Expression::StructConstruct {
                type_name,
                field_values,
                source_location,
            } => {
                // Look up the struct type
                eprintln!("Semantic: Looking up struct type '{}'", type_name.name);

                // Clone the fields to avoid borrowing issues
                let fields_clone = {
                    let type_def = self
                        .symbol_table
                        .lookup_type_definition(&type_name.name)
                        .ok_or_else(|| SemanticError::UndefinedSymbol {
                            symbol: type_name.name.clone(),
                            location: source_location.clone(),
                        })?;

                    // Check that it's a struct type and clone fields
                    if let crate::types::TypeDefinition::Struct { fields, .. } = type_def {
                        fields.clone()
                    } else {
                        return Err(SemanticError::TypeMismatch {
                            expected: "struct type".to_string(),
                            found: "non-struct type".to_string(),
                            location: source_location.clone(),
                        });
                    }
                };

                // Check that all required fields are provided
                for (field_name, _field_type) in &fields_clone {
                    if !field_values
                        .iter()
                        .any(|fv| fv.field_name.name == *field_name)
                    {
                        return Err(SemanticError::MissingField {
                            struct_name: type_name.name.clone(),
                            field_name: field_name.clone(),
                            location: source_location.clone(),
                        });
                    }
                }

                // Check field types
                for field_value in field_values {
                    let expected_type = fields_clone
                        .iter()
                        .find(|(name, _)| name == &field_value.field_name.name)
                        .map(|(_, ty)| ty)
                        .ok_or_else(|| SemanticError::UnknownField {
                            struct_name: type_name.name.clone(),
                            field_name: field_value.field_name.name.clone(),
                            location: field_value.source_location.clone(),
                        })?;

                    let value_type = self.analyze_expression(&field_value.value)?;
                    if !self
                        .type_checker
                        .borrow()
                        .types_compatible(expected_type, &value_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.to_string(),
                            found: value_type.to_string(),
                            location: field_value.source_location.clone(),
                        });
                    }
                }

                // Return the struct type
                Ok(Type::named(
                    type_name.name.clone(),
                    self.current_module.clone(),
                ))
            }

            Expression::FieldAccess {
                instance,
                field_name,
                source_location,
            } => {
                // Get the type of the instance
                let instance_type = self.analyze_expression(instance)?;
                eprintln!(
                    "Semantic: Analyzing FieldAccess. Instance type: {:?}",
                    instance_type
                );

                // Unwrap owned/reference types for field access (auto-deref)
                let mut current_type = &instance_type;
                loop {
                    match current_type {
                        Type::Owned { base_type, .. } => {
                            current_type = base_type;
                        }
                        Type::Pointer { target_type, .. } => {
                            current_type = target_type;
                        }
                        // TODO: Handle Reference if it exists distinct from Pointer
                        _ => {
                            break;
                        }
                    }
                }

                match current_type {
                    Type::Named { name, module } => {
                        // Look up the struct definition
                        let full_name = if let Some(mod_name) = module {
                            format!("{}.{}", mod_name, name)
                        } else {
                            name.clone()
                        };

                        let type_def =
                            self.symbol_table
                                .lookup_type_definition(name)
                                .ok_or_else(|| SemanticError::UndefinedSymbol {
                                    symbol: full_name.clone(),
                                    location: source_location.clone(),
                                })?;

                        // Check that it's a struct and get the field type
                        if let crate::types::TypeDefinition::Struct { fields, .. } = type_def {
                            let field_type = fields
                                .iter()
                                .find(|(fname, _)| fname == &field_name.name)
                                .map(|(_, ftype)| ftype)
                                .ok_or_else(|| SemanticError::UnknownField {
                                    struct_name: name.clone(),
                                    field_name: field_name.name.clone(),
                                    location: source_location.clone(),
                                })?;

                            Ok(field_type.clone())
                        } else {
                            Err(SemanticError::TypeMismatch {
                                expected: "struct type".to_string(),
                                found: instance_type.to_string(),
                                location: source_location.clone(),
                            })
                        }
                    }
                    Type::Array { .. } => {
                        if field_name.name == "length" {
                            Ok(Type::primitive(PrimitiveType::Integer))
                        } else {
                            Err(SemanticError::UnknownField {
                                struct_name: "Array".to_string(),
                                field_name: field_name.name.clone(),
                                location: source_location.clone(),
                            })
                        }
                    }
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "named struct type".to_string(),
                        found: current_type.to_string(),
                        location: source_location.clone(),
                    }),
                }
            }

            Expression::Equals {
                left,
                right,
                source_location,
            } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                // Both operands should be the same type for equality comparison
                if left_type != right_type {
                    return Err(SemanticError::TypeMismatch {
                        expected: left_type.to_string(),
                        found: right_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                // Equality comparison always returns boolean
                Ok(Type::primitive(PrimitiveType::Boolean))
            }

            Expression::NotEquals {
                left,
                right,
                source_location,
            } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                // Both operands should be the same type for inequality comparison
                if left_type != right_type {
                    return Err(SemanticError::TypeMismatch {
                        expected: left_type.to_string(),
                        found: right_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                // Inequality comparison always returns boolean
                Ok(Type::primitive(PrimitiveType::Boolean))
            }

            Expression::EnumVariant {
                enum_name: _,
                variant_name,
                values,
                source_location,
            } => {
                eprintln!(
                    "Semantic: Analyzing enum variant construction: {}",
                    variant_name.name
                );

                // Check if enum name is provided (qualified variant)
                // Note: The parser currently returns enum_name for unqualified variants too (via symbol table lookup?)
                // Actually, parser passes enum_name. If it's unqualified, parser might pass empty?
                // Checking how parser behaves: parse_enum_variant_expression takes enum_name.
                // Wait, unqualified variants are parsed as Identifier first, then maybe converted?
                // Let's assume enum_name is valid or we find it via variant name.

                // In the future, we should improve this by having better variant lookup
                let module_name = self.current_module.clone().unwrap_or_default();
                let enum_type = self
                    .type_checker
                    .borrow()
                    .find_enum_type_by_variant(&variant_name.name, &module_name)
                    .ok_or_else(|| SemanticError::UndefinedSymbol {
                        symbol: format!("enum variant '{}'", variant_name.name),
                        location: source_location.clone(),
                    })?;

                // Check if the variant has an associated value
                let variant = enum_type.get_variant(&variant_name.name).ok_or_else(|| {
                    SemanticError::UndefinedSymbol {
                        symbol: format!("variant '{}' in enum", variant_name.name),
                        location: source_location.clone(),
                    }
                })?;

                // Check argument count
                if variant.associated_types.len() != values.len() {
                    return Err(SemanticError::ArgumentCountMismatch {
                        function: variant_name.name.clone(), // Reusing this error type
                        expected: variant.associated_types.len(),
                        found: values.len(),
                        location: source_location.clone(),
                    });
                }

                // Type check associated values
                for (expected_type, val) in variant.associated_types.iter().zip(values.iter()) {
                    let value_type = self.analyze_expression(val)?;
                    if !self
                        .type_checker
                        .borrow()
                        .types_compatible(expected_type, &value_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.to_string(),
                            found: value_type.to_string(),
                            location: source_location.clone(),
                        });
                    }
                }

                // Return the enum type
                Ok(Type::Named {
                    name: enum_type.name.clone(),
                    module: self.current_module.clone(),
                })
            }

            Expression::Match {
                value,
                cases,
                source_location,
            } => {
                eprintln!("Semantic: Analyzing match expression");

                // Analyze the value being matched
                let value_type = self.analyze_expression(value)?;

                // Ensure it's an enum type
                if !self.type_checker.borrow().is_enum_type(&value_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "enum type".to_string(),
                        found: value_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                // All case expressions must have the same type
                let mut result_type = None;

                for case in cases {
                    // Enter a new scope for pattern bindings
                    self.symbol_table.enter_scope(ScopeKind::Block);

                    // Analyze pattern and set up bindings
                    self.analyze_pattern(&case.pattern, &value_type)?;

                    // Analyze the body expression with pattern bindings in scope
                    let case_type = self.analyze_expression(&case.body)?;

                    // Exit the pattern scope
                    self.symbol_table.exit_scope()?;

                    if let Some(ref expected_type) = result_type {
                        if !self
                            .type_checker
                            .borrow()
                            .are_types_equal(expected_type, &case_type)
                        {
                            return Err(SemanticError::TypeMismatch {
                                expected: expected_type.to_string(),
                                found: case_type.to_string(),
                                location: case.source_location.clone(),
                            });
                        }
                    } else {
                        result_type = Some(case_type);
                    }
                }

                // Check exhaustiveness
                let patterns: Vec<&Pattern> = cases.iter().map(|c| &c.pattern).collect();
                self.check_match_exhaustiveness(&patterns, &value_type, source_location)?;

                result_type.ok_or_else(|| SemanticError::MalformedConstruct {
                    construct: "match expression".to_string(),
                    reason: "no cases provided".to_string(),
                    location: source_location.clone(),
                })
            }

            Expression::TypeCast {
                value,
                target_type,
                failure_behavior: _,
                source_location,
            } => {
                let value_type = self.analyze_expression(value)?;
                let target = self.type_checker.borrow().ast_type_to_type(target_type)?;

                // TODO: Check if the cast is valid
                // For now, we'll allow casts between primitive types
                match (&value_type, &target) {
                    (Type::Primitive(from), Type::Primitive(to)) => {
                        // Allow numeric to string conversions
                        if matches!(to, PrimitiveType::String)
                            && (from.is_numeric() || matches!(from, PrimitiveType::Boolean))
                        {
                            Ok(target)
                        }
                        // Allow string to numeric conversions
                        else if matches!(from, PrimitiveType::String) && to.is_numeric() {
                            Ok(target)
                        }
                        // Allow numeric to numeric conversions
                        else if from.is_numeric() && to.is_numeric() {
                            Ok(target)
                        } else {
                            Err(SemanticError::InvalidOperation {
                                operation: format!("cast from {} to {}", from, to),
                                reason: "invalid type conversion".to_string(),
                                location: source_location.clone(),
                            })
                        }
                    }
                    _ => Err(SemanticError::InvalidOperation {
                        operation: format!("cast from {} to {}", value_type, target),
                        reason: "type casting is only supported for primitive types".to_string(),
                        location: source_location.clone(),
                    }),
                }
            }

            Expression::AddressOf {
                operand,
                mutability,
                source_location,
            } => {
                let operand_type = self.analyze_expression(operand)?;

                // Track the borrow in the symbol table if operand is a variable
                if let Expression::Variable { name, .. } = operand.as_ref() {
                    if *mutability {
                        // Mutable borrow - fails if already borrowed (immutably or mutably)
                        self.symbol_table.borrow_variable_mut(&name.name).map_err(
                            |_| SemanticError::InvalidOperation {
                                operation: "mutable borrow".to_string(),
                                reason: format!(
                                    "cannot borrow '{}' as mutable because it is already borrowed",
                                    name.name
                                ),
                                location: source_location.clone(),
                            },
                        )?;
                    } else {
                        // Immutable borrow - fails if already mutably borrowed
                        self.symbol_table.borrow_variable(&name.name).map_err(
                            |_| SemanticError::InvalidOperation {
                                operation: "immutable borrow".to_string(),
                                reason: format!(
                                    "cannot borrow '{}' as immutable because it is already mutably borrowed",
                                    name.name
                                ),
                                location: source_location.clone(),
                            },
                        )?;
                    }
                }

                // Create a borrowed type (reference) to the operand type
                if *mutability {
                    Ok(Type::Owned {
                        ownership: crate::types::OwnershipKind::MutableBorrow,
                        base_type: Box::new(operand_type),
                    })
                } else {
                    Ok(Type::Owned {
                        ownership: crate::types::OwnershipKind::Borrowed,
                        base_type: Box::new(operand_type),
                    })
                }
            }

            Expression::Dereference {
                pointer,
                source_location,
            } => {
                let pointer_type = self.analyze_expression(pointer)?;
                // Check that it's a pointer type or borrowed reference type
                match pointer_type {
                    Type::Pointer { target_type, .. } => Ok((*target_type).clone()),
                    // Borrowed and mutable borrowed references can also be dereferenced
                    Type::Owned {
                        base_type,
                        ownership: crate::types::OwnershipKind::Borrowed,
                    }
                    | Type::Owned {
                        base_type,
                        ownership: crate::types::OwnershipKind::MutableBorrow,
                    } => Ok((*base_type).clone()),
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "pointer type".to_string(),
                        found: pointer_type.to_string(),
                        location: source_location.clone(),
                    }),
                }
            }

            Expression::PointerArithmetic {
                pointer,
                offset,
                operation: _,
                source_location,
            } => {
                let pointer_type = self.analyze_expression(pointer)?;
                let offset_type = self.analyze_expression(offset)?;

                // Check that first operand is a pointer
                match &pointer_type {
                    Type::Pointer { .. } => {
                        // Check that offset is integer
                        if !matches!(offset_type, Type::Primitive(PrimitiveType::Integer)) {
                            return Err(SemanticError::TypeMismatch {
                                expected: "Integer".to_string(),
                                found: offset_type.to_string(),
                                location: source_location.clone(),
                            });
                        }

                        // Pointer arithmetic returns a pointer of the same type
                        Ok(pointer_type)
                    }
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "pointer type".to_string(),
                        found: pointer_type.to_string(),
                        location: source_location.clone(),
                    }),
                }
            }

            Expression::MethodCall {
                receiver,
                method_name,
                arguments,
                source_location,
            } => {
                let receiver_type = self.analyze_expression(receiver)?;
                let normalized_receiver: Type = match &receiver_type {
                    Type::Owned { base_type, .. } => (**base_type).clone(),
                    _ => receiver_type.clone(),
                };

                match &normalized_receiver {
                    Type::Primitive(PrimitiveType::String) => {
                        if method_name.name == "to_c_string" {
                            if !arguments.is_empty() {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    function: "String.to_c_string".to_string(),
                                    expected: 0,
                                    found: arguments.len(),
                                    location: source_location.clone(),
                                });
                            }
                            // Returns Pointer<Char>
                            Ok(Type::Pointer {
                                target_type: Box::new(Type::Primitive(PrimitiveType::Char)),
                                is_mutable: false,
                            })
                        } else {
                            Err(SemanticError::InvalidOperation {
                                operation: format!("method call '{}'", method_name.name),
                                reason: "method not found on type String".to_string(),
                                location: source_location.clone(),
                            })
                        }
                    }
                    Type::Array { element_type, .. } => {
                        if method_name.name == "as_ptr" {
                            if !arguments.is_empty() {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    function: "Array.as_ptr".to_string(),
                                    expected: 0,
                                    found: arguments.len(),
                                    location: source_location.clone(),
                                });
                            }
                            // Returns Pointer<T>
                            Ok(Type::Pointer {
                                target_type: element_type.clone(),
                                is_mutable: true, 
                            })
                        } else {
                            Err(SemanticError::InvalidOperation {
                                operation: format!("method call '{}'", method_name.name),
                                reason: "method not found on type Array".to_string(),
                                location: source_location.clone(),
                            })
                        }
                    }
                    Type::Map {
                        key_type,
                        value_type,
                    } => {
                        if method_name.name == "insert" {
                            if arguments.len() != 2 {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    function: "map.insert".to_string(),
                                    expected: 2,
                                    found: arguments.len(),
                                    location: source_location.clone(),
                                });
                            }

                            // Check key type
                            let arg_key_type = self.analyze_expression(&arguments[0].value)?;
                            if !self
                                .type_checker
                                .borrow()
                                .types_compatible(key_type, &arg_key_type)
                            {
                                return Err(SemanticError::TypeMismatch {
                                    expected: key_type.to_string(),
                                    found: arg_key_type.to_string(),
                                    location: arguments[0].source_location.clone(),
                                });
                            }

                            // Check value type
                            let arg_value_type = self.analyze_expression(&arguments[1].value)?;
                            if !self
                                .type_checker
                                .borrow()
                                .types_compatible(value_type, &arg_value_type)
                            {
                                return Err(SemanticError::TypeMismatch {
                                    expected: value_type.to_string(),
                                    found: arg_value_type.to_string(),
                                    location: arguments[1].source_location.clone(),
                                });
                            }

                            Ok(Type::Primitive(PrimitiveType::Void))
                        } else if method_name.name == "get" {
                            if arguments.len() != 1 {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    function: "map.get".to_string(),
                                    expected: 1,
                                    found: arguments.len(),
                                    location: source_location.clone(),
                                });
                            }

                            // Check key type
                            let arg_key_type = self.analyze_expression(&arguments[0].value)?;
                            if !self
                                .type_checker
                                .borrow()
                                .types_compatible(key_type, &arg_key_type)
                            {
                                return Err(SemanticError::TypeMismatch {
                                    expected: key_type.to_string(),
                                    found: arg_key_type.to_string(),
                                    location: arguments[0].source_location.clone(),
                                });
                            }

                            // Returns value type
                            Ok(*value_type.clone())
                        } else {
                            Err(SemanticError::InvalidOperation {
                                operation: format!("method call '{}'", method_name.name),
                                reason: "method not found on type Map".to_string(),
                                location: source_location.clone(),
                            })
                        }
                    }
                    Type::Module(module_name) => {
                        // Use receiver's name if possible (to handle aliasing)
                        let module_prefix = if let Expression::Variable { name, .. } = &**receiver {
                            name.name.clone()
                        } else {
                            module_name.clone()
                        };

                        let qualified_name = format!("{}.{}", module_prefix, method_name.name);

                        // Clone the function type info to avoid holding a borrow on symbol_table
                        let (parameter_types, return_type) = if let Some(symbol) =
                            self.symbol_table.lookup_symbol(&qualified_name)
                        {
                            if let Type::Function {
                                parameter_types,
                                return_type,
                                is_variadic: _,
                            } = &symbol.symbol_type
                            {
                                (parameter_types.clone(), return_type.clone())
                            } else {
                                return Err(SemanticError::InvalidOperation {
                                    operation: format!("call '{}'", qualified_name),
                                    reason: "symbol is not a function".to_string(),
                                    location: source_location.clone(),
                                });
                            }
                        } else {
                            return Err(SemanticError::UndefinedSymbol {
                                symbol: qualified_name,
                                location: source_location.clone(),
                            });
                        };

                        // Check arguments
                        if parameter_types.len() != arguments.len() {
                            return Err(SemanticError::ArgumentCountMismatch {
                                function: qualified_name,
                                expected: parameter_types.len(),
                                found: arguments.len(),
                                location: source_location.clone(),
                            });
                        }

                        for (param_type, arg) in parameter_types.iter().zip(arguments.iter()) {
                            let arg_type = self.analyze_expression(&arg.value)?;
                            if !self
                                .type_checker
                                .borrow()
                                .types_compatible(param_type, &arg_type)
                            {
                                return Err(SemanticError::TypeMismatch {
                                    expected: param_type.to_string(),
                                    found: arg_type.to_string(),
                                    location: arg.source_location.clone(),
                                });
                            }
                        }

                        Ok(*return_type)
                    }
                    _ => {
                        if let Some(return_ty) = self.resolve_trait_method_call(
                            &receiver_type,
                            method_name,
                            arguments,
                            source_location,
                        )? {
                            Ok(return_ty)
                        } else {
                            Err(SemanticError::InvalidOperation {
                                operation: format!("method call '{}'", method_name.name),
                                reason: format!(
                                    "type '{}' does not support methods",
                                    receiver_type
                                ),
                                location: source_location.clone(),
                            })
                        }
                    }
                }
            }

            Expression::MapLiteral {
                key_type,
                value_type,
                entries,
                source_location: _,
            } => {
                // Convert AST types to semantic types
                let key_sem_type = self.type_checker.borrow().ast_type_to_type(key_type)?;
                let value_sem_type = self.type_checker.borrow().ast_type_to_type(value_type)?;

                // Check all entries match the declared types
                for entry in entries {
                    let entry_key_type = self.analyze_expression(&entry.key)?;
                    let entry_value_type = self.analyze_expression(&entry.value)?;

                    if !self
                        .type_checker
                        .borrow()
                        .types_compatible(&key_sem_type, &entry_key_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: key_sem_type.to_string(),
                            found: entry_key_type.to_string(),
                            location: entry.source_location.clone(),
                        });
                    }

                    if !self
                        .type_checker
                        .borrow()
                        .types_compatible(&value_sem_type, &entry_value_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: value_sem_type.to_string(),
                            found: entry_value_type.to_string(),
                            location: entry.source_location.clone(),
                        });
                    }
                }

                Ok(Type::map(key_sem_type, value_sem_type))
            }

            Expression::MapAccess {
                map,
                key,
                source_location,
            } => {
                let map_type = self.analyze_expression(map)?;

                // Check that it's a map
                match map_type {
                    Type::Map {
                        key_type,
                        value_type,
                    } => {
                        // Check key type
                        let provided_key_type = self.analyze_expression(key)?;
                        if !self
                            .type_checker
                            .borrow()
                            .types_compatible(&key_type, &provided_key_type)
                        {
                            return Err(SemanticError::TypeMismatch {
                                expected: key_type.to_string(),
                                found: provided_key_type.to_string(),
                                location: source_location.clone(),
                            });
                        }

                        Ok((*value_type).clone())
                    }
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "Map".to_string(),
                        found: map_type.to_string(),
                        location: source_location.clone(),
                    }),
                }
            }

            Expression::LessThan {
                left,
                right,
                source_location,
            }
            | Expression::LessThanOrEqual {
                left,
                right,
                source_location,
            }
            | Expression::GreaterThan {
                left,
                right,
                source_location,
            }
            | Expression::GreaterThanOrEqual {
                left,
                right,
                source_location,
            } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                // Check types are compatible for comparison
                if self
                    .type_checker
                    .borrow()
                    .types_compatible(&left_type, &right_type)
                {
                    Ok(Type::primitive(PrimitiveType::Boolean))
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: left_type.to_string(),
                        found: right_type.to_string(),
                        location: source_location.clone(),
                    })
                }
            }

            Expression::Negate {
                operand,
                source_location,
            } => {
                let operand_type = self.analyze_expression(operand)?;

                // Operand must be numeric (integer or float)
                match &operand_type {
                    Type::Primitive(PrimitiveType::Integer)
                    | Type::Primitive(PrimitiveType::Integer32)
                    | Type::Primitive(PrimitiveType::Integer64)
                    | Type::Primitive(PrimitiveType::Float)
                    | Type::Primitive(PrimitiveType::Float32)
                    | Type::Primitive(PrimitiveType::Float64) => Ok(operand_type),
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "numeric type".to_string(),
                        found: operand_type.to_string(),
                        location: source_location.clone(),
                    }),
                }
            }

            Expression::LogicalNot {
                operand,
                source_location,
            } => {
                let operand_type = self.analyze_expression(operand)?;

                // Operand must be boolean
                if !matches!(operand_type, Type::Primitive(PrimitiveType::Boolean)) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Boolean".to_string(),
                        found: operand_type.to_string(),
                        location: source_location.clone(),
                    });
                }

                Ok(Type::primitive(PrimitiveType::Boolean))
            }

            Expression::LogicalAnd {
                operands,
                source_location,
            }
            | Expression::LogicalOr {
                operands,
                source_location,
            } => {
                // All operands must be boolean
                for operand in operands {
                    let operand_type = self.analyze_expression(operand)?;
                    if !matches!(operand_type, Type::Primitive(PrimitiveType::Boolean)) {
                        return Err(SemanticError::TypeMismatch {
                            expected: "Boolean".to_string(),
                            found: operand_type.to_string(),
                            location: source_location.clone(),
                        });
                    }
                }

                Ok(Type::primitive(PrimitiveType::Boolean))
            }

            Expression::Lambda {
                captures,
                parameters,
                return_type,
                body,
                source_location,
            } => {
                // First, look up all captures in the CURRENT (parent) scope before entering lambda scope
                let mut captured_symbols = Vec::new();
                for capture in captures {
                    // Look up the captured variable in the enclosing scope
                    if let Some(symbol) = self.symbol_table.lookup_symbol(&capture.name.name) {
                        captured_symbols.push(symbol.clone());
                    } else {
                        return Err(SemanticError::UndefinedSymbol {
                            symbol: capture.name.name.clone(),
                            location: capture.source_location.clone(),
                        });
                    }
                }

                // Enter a new scope for the lambda
                self.symbol_table.enter_scope(ScopeKind::Function);

                // Add captured variables to the lambda scope
                for symbol in captured_symbols {
                    let capture_symbol = Symbol {
                        name: symbol.name.clone(),
                        symbol_type: symbol.symbol_type.clone(),
                        kind: SymbolKind::Variable, // Captured as a local variable
                        is_mutable: false,          // Captures are immutable by default
                        is_initialized: true,
                        declaration_location: symbol.declaration_location.clone(),
                        is_moved: false,
                        borrow_state: BorrowState::None,
                        ffi_symbol: None,
                    };
                    self.symbol_table.add_symbol(capture_symbol)?;
                }

                // Add parameters to lambda scope
                let mut param_types = Vec::new();
                for param in parameters {
                    let param_type = self
                        .type_checker
                        .borrow()
                        .ast_type_to_type(&param.param_type)?;
                    param_types.push(param_type.clone());

                    let param_symbol = Symbol {
                        name: param.name.name.clone(),
                        symbol_type: param_type,
                        kind: SymbolKind::Parameter,
                        is_mutable: false,
                        is_initialized: true,
                        declaration_location: param.source_location.clone(),
                        is_moved: false,
                        borrow_state: BorrowState::None,
                        ffi_symbol: None,
                    };
                    self.symbol_table.add_symbol(param_symbol)?;
                }

                // Analyze the lambda body and determine return type
                let body_type = match body {
                    LambdaBody::Expression(expr) => self.analyze_expression(expr)?,
                    LambdaBody::Block(block) => {
                        // Analyze block and infer return type from return statements
                        self.analyze_lambda_block_return_type(block)?
                    }
                };

                // Determine the return type
                let lambda_return_type = if let Some(explicit_return) = return_type {
                    let explicit_type = self
                        .type_checker
                        .borrow()
                        .ast_type_to_type(explicit_return)?;
                    // Check that body type matches explicit return type
                    if !self
                        .type_checker
                        .borrow()
                        .are_types_equal(&explicit_type, &body_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: explicit_type.to_string(),
                            found: body_type.to_string(),
                            location: source_location.clone(),
                        });
                    }
                    explicit_type
                } else {
                    // Infer return type from body
                    body_type
                };

                // Exit lambda scope
                self.symbol_table.exit_scope()?;

                // Return a function type
                Ok(Type::Function {
                    parameter_types: param_types,
                    return_type: Box::new(lambda_return_type),
                    is_variadic: false,
                })
            }

            // TODO: Handle other expression types
            _ => {
                eprintln!("Warning: Unhandled expression type in semantic analysis");
                // For unimplemented expressions, return error type
                Ok(Type::Error)
            }
        }
    }

    /// Analyze an assignment target
    fn analyze_assignment_target(
        &mut self,
        target: &AssignmentTarget,
    ) -> Result<Type, SemanticError> {
        match target {
            AssignmentTarget::Variable { name } => {
                let symbol = self.symbol_table.lookup_symbol(&name.name).ok_or_else(|| {
                    SemanticError::UndefinedSymbol {
                        symbol: name.name.clone(),
                        location: SourceLocation::unknown(), // TODO: Better location tracking
                    }
                })?;

                Ok(symbol.symbol_type.clone())
            }

            AssignmentTarget::MapValue { map, key } => {
                let map_type = self.analyze_expression(map)?;

                // Check that it's a map
                match map_type {
                    Type::Map {
                        key_type,
                        value_type,
                    } => {
                        // Check key type
                        let provided_key_type = self.analyze_expression(key)?;
                        if !self
                            .type_checker
                            .borrow()
                            .types_compatible(&key_type, &provided_key_type)
                        {
                            return Err(SemanticError::TypeMismatch {
                                expected: key_type.to_string(),
                                found: provided_key_type.to_string(),
                                location: SourceLocation::unknown(),
                            });
                        }

                        Ok((*value_type).clone())
                    }
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "Map".to_string(),
                        found: map_type.to_string(),
                        location: SourceLocation::unknown(),
                    }),
                }
            }

            // TODO: Handle other assignment targets
            _ => Ok(Type::Error),
        }
    }

    /// Analyze a function call
    fn analyze_function_call(&mut self, call: &FunctionCall) -> Result<Type, SemanticError> {
        match &call.function_reference {
            FunctionReference::Local { name } => {
                // Check for built-in functions first
                if name.name == "printf" {
                    // printf returns int
                    return Ok(Type::primitive(PrimitiveType::Integer));
                }

                // Clone the function type to avoid borrowing issues
                let (return_type, parameter_types) = {
                    let symbol = self.symbol_table.lookup_symbol(&name.name).ok_or_else(|| {
                        SemanticError::UndefinedSymbol {
                            symbol: name.name.clone(),
                            location: SourceLocation::unknown(), // TODO: Better location tracking
                        }
                    })?;

                    // Extract return type from function type
                    if let Type::Function {
                        return_type,
                        parameter_types,
                        is_variadic: _,
                    } = &symbol.symbol_type
                    {
                        ((**return_type).clone(), parameter_types.clone())
                    } else {
                        return Err(SemanticError::TypeMismatch {
                            expected: "function type".to_string(),
                            found: symbol.symbol_type.to_string(),
                            location: SourceLocation::unknown(),
                        });
                    }
                };

                // Check argument count - include both named and variadic arguments
                let total_args = call.arguments.len() + call.variadic_arguments.len();
                if total_args != parameter_types.len() {
                    return Err(SemanticError::ArgumentCountMismatch {
                        function: name.name.clone(),
                        expected: parameter_types.len(),
                        found: total_args,
                        location: SourceLocation::unknown(),
                    });
                }

                // Check ownership transfers for each argument
                for (i, arg) in call.arguments.iter().enumerate() {
                    let arg_type = self.analyze_expression(&arg.value)?;

                    if let Some(param_type) = parameter_types.get(i) {
                        if !self
                            .type_checker
                            .borrow()
                            .types_compatible(param_type, &arg_type)
                        {
                            return Err(SemanticError::TypeMismatch {
                                expected: param_type.to_string(),
                                found: arg_type.to_string(),
                                location: arg.source_location.clone(),
                            });
                        }

                        // Check ownership transfer
                        self.check_argument_ownership(&arg.value, param_type)?;
                    }
                }

                // Handle variadic arguments
                for (i, arg) in call.variadic_arguments.iter().enumerate() {
                    let arg_type = self.analyze_expression(arg.as_ref())?;
                    let param_index = call.arguments.len() + i;

                    if let Some(param_type) = parameter_types.get(param_index) {
                        if !self
                            .type_checker
                            .borrow()
                            .types_compatible(param_type, &arg_type)
                        {
                            return Err(SemanticError::TypeMismatch {
                                expected: param_type.to_string(),
                                found: arg_type.to_string(),
                                location: SourceLocation::unknown(),
                            });
                        }

                        // Check ownership transfer for variadic args that happen to match a parameter
                        // (e.g. if parameter_types includes them but they were passed as variadic in AST?)
                        // Note: usually parameter_types only has fixed args.
                        self.check_argument_ownership(arg.as_ref(), param_type)?;
                    }
                }

                Ok(return_type)
            }

            // TODO: Handle qualified and external function references
            _ => Ok(Type::Error),
        }
    }

    /// Resolve a function reference to a symbol
    /// Note: Currently unused, but retained for future use in trait method resolution
    #[allow(dead_code)]
    fn resolve_function(
        &self,
        reference: &FunctionReference,
        source_location: &SourceLocation,
    ) -> Result<Symbol, SemanticError> {
        match reference {
            FunctionReference::Local { name } => self
                .symbol_table
                .lookup_symbol(&name.name)
                .cloned()
                .ok_or_else(|| SemanticError::UndefinedSymbol {
                    symbol: name.name.clone(),
                    location: source_location.clone(),
                }),
            FunctionReference::External { name } => self
                .symbol_table
                .lookup_symbol(&name.name)
                .cloned()
                .ok_or_else(|| SemanticError::UndefinedSymbol {
                    symbol: name.name.clone(),
                    location: source_location.clone(),
                }),
            FunctionReference::Qualified { module, name } => {
                // TODO: Implement qualified lookup
                // For now, try fully qualified name "module.name"
                let qualified_name = format!("{}.{}", module.name, name.name);
                self.symbol_table
                    .lookup_symbol(&qualified_name)
                    .cloned()
                    .ok_or_else(|| SemanticError::UndefinedSymbol {
                        symbol: qualified_name,
                        location: source_location.clone(),
                    })
            }
        }
    }

    fn analyze_function_call_expression(
        &mut self,
        call: &FunctionCall,
        source_location: &SourceLocation,
    ) -> Result<Type, SemanticError> {
        let function_name = match &call.function_reference {
            FunctionReference::Local { name } => name.name.clone(),
            FunctionReference::Qualified { module, name } => {
                format!("{}.{}", module.name, name.name)
            }
            FunctionReference::External { name } => name.name.clone(),
        };

        // First, try to find the AST function to get its generic parameters
        let func_ast_opt = self.lookup_ast_function_in_modules(&function_name);

        let (instantiated_func_type, _generic_param_names) = if let Some(func_ast) = func_ast_opt {
            // This is a generic function
            let ast_generic_params = &func_ast.generic_parameters;
            let explicit_type_args = &call.explicit_type_arguments;

            // 1. Validate generic argument count
            if ast_generic_params.len() != explicit_type_args.len() {
                return Err(SemanticError::GenericArgumentCountMismatch {
                    function: function_name.clone(),
                    expected: ast_generic_params.len(),
                    found: explicit_type_args.len(),
                    location: source_location.clone(),
                });
            }

            // 2. Build type map from generic parameter names to concrete types
            let mut type_map = HashMap::new();
            let mut generic_param_names_vec = Vec::new();
            for (gen_param, explicit_arg_spec) in
                ast_generic_params.iter().zip(explicit_type_args.iter())
            {
                let explicit_type = self
                    .type_checker
                    .borrow()
                    .ast_type_to_type(explicit_arg_spec)?;
                type_map.insert(gen_param.name.name.clone(), explicit_type);
                generic_param_names_vec.push(gen_param.name.name.clone());
            }

            // 3. Get the base function type from the symbol table
            let symbol = self
                .symbol_table
                .lookup_symbol(&function_name)
                .ok_or_else(|| SemanticError::UndefinedSymbol {
                    symbol: function_name.clone(),
                    location: source_location.clone(),
                })?;

            let base_func_type = match &symbol.symbol_type {
                Type::Function {
                    parameter_types,
                    return_type,
                    is_variadic,
                } => Type::Function {
                    parameter_types: parameter_types.clone(),
                    return_type: return_type.clone(),
                    is_variadic: *is_variadic,
                },
                _ => {
                    return Err(SemanticError::TypeMismatch {
                        expected: "function type".to_string(),
                        found: symbol.symbol_type.to_string(),
                        location: source_location.clone(),
                    });
                }
            };

            // 4. Substitute generic types in the function type
            let substituted_func_type = self.substitute_type(&base_func_type, &type_map);
            (substituted_func_type, Some(generic_param_names_vec))
        } else {
            // Not a generic function or AST not found, proceed with direct lookup
            if !call.explicit_type_arguments.is_empty() {
                return Err(SemanticError::GenericArgumentCountMismatch {
                    function: function_name.clone(),
                    expected: 0,
                    found: call.explicit_type_arguments.len(),
                    location: source_location.clone(),
                });
            }

            let symbol = self
                .symbol_table
                .lookup_symbol(&function_name)
                .ok_or_else(|| SemanticError::UndefinedSymbol {
                    symbol: function_name.clone(),
                    location: source_location.clone(),
                })?;

            let func_type = match &symbol.symbol_type {
                Type::Function {
                    parameter_types,
                    return_type,
                    is_variadic,
                } => Type::Function {
                    parameter_types: parameter_types.clone(),
                    return_type: return_type.clone(),
                    is_variadic: *is_variadic,
                },
                _ => {
                    return Err(SemanticError::TypeMismatch {
                        expected: "function type".to_string(),
                        found: symbol.symbol_type.to_string(),
                        location: source_location.clone(),
                    });
                }
            };
            (func_type, None)
        };

        // Continue with argument type checking using the instantiated_func_type
        let expected_param_count = if let Type::Function {
            parameter_types, ..
        } = &instantiated_func_type
        {
            parameter_types.len()
        } else {
            0
        };

        let provided_arg_count = call.arguments.len();

        if !instantiated_func_type.is_variadic() && provided_arg_count != expected_param_count {
            return Err(SemanticError::ArgumentCountMismatch {
                function: function_name.clone(),
                expected: expected_param_count,
                found: provided_arg_count,
                location: source_location.clone(),
            });
        }

        // Analyze arguments and check types
        if let Type::Function {
            parameter_types, ..
        } = &instantiated_func_type
        {
            for (i, arg) in call.arguments.iter().enumerate() {
                let arg_type = self.analyze_expression(&arg.value)?;

                if i < parameter_types.len() {
                    let expected_type = &parameter_types[i];
                    if !self
                        .type_checker
                        .borrow()
                        .types_compatible(expected_type, &arg_type)
                    {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.to_string(),
                            found: arg_type.to_string(),
                            location: arg.source_location.clone(),
                        });
                    }
                    // Check ownership transfer (move/borrow)
                    self.check_argument_ownership(&arg.value, expected_type)?;
                } else if !instantiated_func_type.is_variadic() {
                    // This case should be caught by argument count mismatch earlier, but defensive check
                    return Err(SemanticError::ArgumentCountMismatch {
                        function: function_name.clone(),
                        expected: expected_param_count,
                        found: provided_arg_count,
                        location: source_location.clone(),
                    });
                }
            }
        }

        // Analyze variadic arguments
        for arg_expr in &call.variadic_arguments {
            let _ = self.analyze_expression(arg_expr)?;
            // Variadic arguments are not type-checked against a specific parameter type
            // They are passed as-is to the runtime/FFI.
        }

        if let Type::Function { return_type, .. } = instantiated_func_type {
            Ok(*return_type)
        } else {
            // Should not happen due to match above
            Err(SemanticError::Internal {
                message: "Function symbol did not resolve to a function type after instantiation"
                    .to_string(),
            })
        }
    }

    /// Look up a trait method signature using a trait bound only (no concrete impl)
    fn trait_method_signature_from_trait(
        &self,
        trait_name: &str,
        method_name: &str,
        self_type: &Type,
    ) -> Result<Option<MethodSignatureInfo>, SemanticError> {
        let trait_def = match self.trait_definitions.get(trait_name) {
            Some(def) => def,
            None => return Ok(None),
        };

        let substitutions = HashMap::new();
        for method in &trait_def.methods {
            if method.name.name == method_name {
                let sig = self.compute_method_signature(
                    &method.parameters,
                    &method.return_type,
                    &trait_def.generic_parameters,
                    &method.generic_parameters,
                    self_type,
                    &substitutions,
                )?;
                return Ok(Some(sig));
            }
        }

        Ok(None)
    }

    /// Attempt to resolve a trait method call on a generic receiver using its constraints
    fn resolve_trait_method_on_generic(
        &mut self,
        receiver_type: &Type,
        normalized_receiver: &Type,
        method_name: &str,
        arguments: &[Argument],
        source_location: &SourceLocation,
    ) -> Result<Option<Type>, SemanticError> {
        if let Type::Generic {
            constraints, name, ..
        } = normalized_receiver
        {
            let mut all_constraints: Vec<crate::types::TypeConstraintInfo> = constraints.clone();

            // Merge in constraints from current function where-clause if present
            if let Some(extra) = self.current_generic_constraints.get(name) {
                all_constraints.extend_from_slice(extra);
            }

            for constraint in all_constraints {
                if let crate::types::TypeConstraintInfo::TraitBound { trait_name, .. } = constraint
                {
                    if let Some(sig) = self.trait_method_signature_from_trait(
                        &trait_name,
                        method_name,
                        receiver_type,
                    )? {
                        if arguments.len() != sig.call_param_types.len() {
                            return Err(SemanticError::ArgumentCountMismatch {
                                function: method_name.to_string(),
                                expected: sig.call_param_types.len(),
                                found: arguments.len(),
                                location: source_location.clone(),
                            });
                        }

                        if let Some(self_ty) = sig.self_param_type {
                            if !self
                                .type_checker
                                .borrow()
                                .types_compatible(&self_ty, receiver_type)
                            {
                                return Err(SemanticError::TypeMismatch {
                                    expected: self_ty.to_string(),
                                    found: receiver_type.to_string(),
                                    location: source_location.clone(),
                                });
                            }
                        }

                        for (arg, expected_ty) in arguments.iter().zip(sig.call_param_types.iter())
                        {
                            let actual_ty = self.analyze_expression(&arg.value)?;
                            if !self
                                .type_checker
                                .borrow()
                                .types_compatible(expected_ty, &actual_ty)
                            {
                                return Err(SemanticError::TypeMismatch {
                                    expected: expected_ty.to_string(),
                                    found: actual_ty.to_string(),
                                    location: arg.source_location.clone(),
                                });
                            }
                        }

                        return Ok(Some(sig.return_type));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Resolve trait-based method calls using dispatch table or generic bounds
    fn resolve_trait_method_call(
        &mut self,
        receiver_type: &Type,
        method_name: &Identifier,
        arguments: &[Argument],
        source_location: &SourceLocation,
    ) -> Result<Option<Type>, SemanticError> {
        let normalized_receiver: Type = match receiver_type {
            Type::Owned { base_type, .. } => (**base_type).clone(),
            _ => receiver_type.clone(),
        };

        let key = TraitMethodKey {
            receiver: normalized_receiver.clone(),
            method_name: method_name.name.clone(),
        };

        if let Some(dispatch) = self.trait_dispatch_table.get(&key).cloned() {
            if let Some(self_ty) = &dispatch.self_param_type {
                if !self
                    .type_checker
                    .borrow()
                    .types_compatible(self_ty, receiver_type)
                {
                    return Err(SemanticError::TypeMismatch {
                        expected: self_ty.to_string(),
                        found: receiver_type.to_string(),
                        location: source_location.clone(),
                    });
                }
            }

            if arguments.len() != dispatch.param_types.len() {
                return Err(SemanticError::ArgumentCountMismatch {
                    function: method_name.name.clone(),
                    expected: dispatch.param_types.len(),
                    found: arguments.len(),
                    location: source_location.clone(),
                });
            }

            for (arg, expected_ty) in arguments.iter().zip(dispatch.param_types.iter()) {
                let actual_ty = self.analyze_expression(&arg.value)?;
                if !self
                    .type_checker
                    .borrow()
                    .types_compatible(expected_ty, &actual_ty)
                {
                    return Err(SemanticError::TypeMismatch {
                        expected: expected_ty.to_string(),
                        found: actual_ty.to_string(),
                        location: arg.source_location.clone(),
                    });
                }
            }

            return Ok(Some(dispatch.return_type.clone()));
        }

        if let Some(return_type) = self.resolve_trait_method_on_generic(
            receiver_type,
            &normalized_receiver,
            &method_name.name,
            arguments,
            source_location,
        )? {
            return Ok(Some(return_type));
        }

        Ok(None)
    }

    /// Analyze an if statement
    fn analyze_if_statement(
        &mut self,
        condition: &Expression,
        then_block: &Block,
        else_ifs: &[ElseIf],
        else_block: &Option<Block>,
    ) -> Result<(), SemanticError> {
        // Analyze condition - must be boolean
        let condition_type = self.analyze_expression(condition)?;
        if !matches!(
            condition_type,
            Type::Primitive(PrimitiveType::Boolean) | Type::Error
        ) {
            return Err(SemanticError::TypeMismatch {
                expected: "Boolean".to_string(),
                found: condition_type.to_string(),
                location: SourceLocation::unknown(), // TODO: Better location tracking
            });
        }

        // Analyze then block
        self.analyze_block(then_block)?;

        // Analyze else-if blocks
        for else_if in else_ifs {
            let else_if_condition_type = self.analyze_expression(&else_if.condition)?;
            if !matches!(
                else_if_condition_type,
                Type::Primitive(PrimitiveType::Boolean) | Type::Error
            ) {
                return Err(SemanticError::TypeMismatch {
                    expected: "Boolean".to_string(),
                    found: else_if_condition_type.to_string(),
                    location: else_if.source_location.clone(),
                });
            }
            self.analyze_block(&else_if.block)?;
        }

        // Analyze else block if present
        if let Some(else_block) = else_block {
            self.analyze_block(else_block)?;
        }

        Ok(())
    }

    /// Analyze a while loop
    fn analyze_while_loop(
        &mut self,
        condition: &Expression,
        body: &Block,
        invariant: &Option<String>,
    ) -> Result<(), SemanticError> {
        // Analyze condition - must be boolean
        let condition_type = self.analyze_expression(condition)?;
        if !matches!(
            condition_type,
            Type::Primitive(PrimitiveType::Boolean) | Type::Error
        ) {
            return Err(SemanticError::TypeMismatch {
                expected: "Boolean".to_string(),
                found: condition_type.to_string(),
                location: SourceLocation::unknown(),
            });
        }

        // TODO: Process invariant for formal verification
        if let Some(_invariant_str) = invariant {
            // Future: Parse and validate invariant expression
        }

        // Enter loop scope
        self.symbol_table.enter_scope(ScopeKind::Loop);

        // Analyze loop body
        self.analyze_block(body)?;

        // Exit loop scope
        self.symbol_table.exit_scope()?;

        Ok(())
    }

    /// Analyze a for-each loop
    fn analyze_for_each_loop(
        &mut self,
        collection: &Expression,
        element_binding: &Identifier,
        element_type: &TypeSpecifier,
        body: &Block,
    ) -> Result<(), SemanticError> {
        // Analyze collection expression
        let collection_type = self.analyze_expression(collection)?;

        // Check that collection is iterable (array or map)
        let element_actual_type = match &collection_type {
            Type::Array { element_type, .. } => (**element_type).clone(),
            Type::Map { value_type, .. } => (**value_type).clone(),
            _ => {
                return Err(SemanticError::TypeMismatch {
                    expected: "Array or Map".to_string(),
                    found: collection_type.to_string(),
                    location: SourceLocation::unknown(),
                });
            }
        };

        // Check element type compatibility
        let declared_element_type = self.type_checker.borrow().ast_type_to_type(element_type)?;
        if !self
            .type_checker
            .borrow()
            .types_compatible(&declared_element_type, &element_actual_type)
        {
            return Err(SemanticError::TypeMismatch {
                expected: declared_element_type.to_string(),
                found: element_actual_type.to_string(),
                location: element_binding.source_location.clone(),
            });
        }

        // Enter loop scope
        self.symbol_table.enter_scope(ScopeKind::Loop);

        // Add element binding to scope
        let element_symbol = Symbol {
            name: element_binding.name.clone(),
            symbol_type: declared_element_type,
            kind: SymbolKind::Variable,
            is_mutable: false, // Loop variables are typically immutable
            is_initialized: true,
            declaration_location: element_binding.source_location.clone(),
            is_moved: false,
            borrow_state: BorrowState::None,
            ffi_symbol: None,
        };
        self.symbol_table.add_symbol(element_symbol)?;

        // Analyze loop body
        self.analyze_block(body)?;

        // Exit loop scope
        self.symbol_table.exit_scope()?;

        Ok(())
    }

    /// Analyze a fixed iteration loop
    fn analyze_fixed_iteration_loop(
        &mut self,
        counter: &Identifier,
        from_value: &Expression,
        to_value: &Expression,
        step_value: &Option<Box<Expression>>,
        body: &Block,
    ) -> Result<(), SemanticError> {
        // Analyze from and to expressions - must be numeric
        let from_type = self.analyze_expression(from_value)?;
        if !from_type.is_numeric() {
            return Err(SemanticError::TypeMismatch {
                expected: "numeric type".to_string(),
                found: from_type.to_string(),
                location: SourceLocation::unknown(),
            });
        }

        let to_type = self.analyze_expression(to_value)?;
        if !to_type.is_numeric() {
            return Err(SemanticError::TypeMismatch {
                expected: "numeric type".to_string(),
                found: to_type.to_string(),
                location: SourceLocation::unknown(),
            });
        }

        // Analyze step value if present
        if let Some(step) = step_value {
            let step_type = self.analyze_expression(step)?;
            if !step_type.is_numeric() {
                return Err(SemanticError::TypeMismatch {
                    expected: "numeric type".to_string(),
                    found: step_type.to_string(),
                    location: SourceLocation::unknown(),
                });
            }
        }

        // Enter loop scope
        self.symbol_table.enter_scope(ScopeKind::Loop);

        // Add counter variable to scope
        let counter_symbol = Symbol {
            name: counter.name.clone(),
            symbol_type: Type::primitive(PrimitiveType::Integer),
            kind: SymbolKind::Variable,
            is_mutable: false, // Loop counter is immutable within the loop
            is_initialized: true,
            declaration_location: counter.source_location.clone(),
            is_moved: false,
            borrow_state: BorrowState::None,
            ffi_symbol: None,
        };
        self.symbol_table.add_symbol(counter_symbol)?;

        // Analyze loop body
        self.analyze_block(body)?;

        // Exit loop scope
        self.symbol_table.exit_scope()?;

        Ok(())
    }

    /// Analyze a break statement
    fn analyze_break_statement(
        &mut self,
        target_label: &Option<Identifier>,
        source_location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // TODO: Check that we're inside a loop
        // TODO: If label is specified, check that it matches a loop label
        if target_label.is_some() {
            // Future: Implement labeled loop tracking
            return Err(SemanticError::UnsupportedFeature {
                feature: "labeled break".to_string(),
                location: source_location.clone(),
            });
        }

        Ok(())
    }

    /// Analyze a continue statement
    fn analyze_continue_statement(
        &mut self,
        target_label: &Option<Identifier>,
        source_location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // TODO: Check that we're inside a loop
        // TODO: If label is specified, check that it matches a loop label
        if target_label.is_some() {
            // Future: Implement labeled loop tracking
            return Err(SemanticError::UnsupportedFeature {
                feature: "labeled continue".to_string(),
                location: source_location.clone(),
            });
        }

        Ok(())
    }

    /// Analyze a try-catch block
    fn analyze_try_block(
        &mut self,
        protected_block: &Block,
        catch_clauses: &[CatchClause],
        finally_block: &Option<Block>,
    ) -> Result<(), SemanticError> {
        // Track exception flow - save current state
        let saved_exceptions = self.current_exceptions.clone();
        let mut caught_exception_types = Vec::new();

        // Analyze protected block with exception tracking
        self.analyze_block(protected_block)?;

        // Validate catch clauses
        for catch in catch_clauses {
            // Validate exception type exists and is throwable
            let exception_type = self
                .type_checker
                .borrow()
                .ast_type_to_type(&catch.exception_type)?;

            // Check for duplicate catch clauses
            if caught_exception_types.contains(&exception_type) {
                return Err(SemanticError::DuplicateCatchClause {
                    exception_type: format!("{:?}", exception_type),
                    location: catch.handler_block.source_location.clone(),
                });
            }
            caught_exception_types.push(exception_type.clone());

            // Enter catch block scope
            self.symbol_table.enter_scope(ScopeKind::Block);

            // Add exception binding if present
            if let Some(binding) = &catch.binding_variable {
                let exception_symbol = Symbol {
                    name: binding.name.clone(),
                    symbol_type: exception_type.clone(),
                    kind: SymbolKind::Variable,
                    is_mutable: false,
                    is_initialized: true,
                    declaration_location: binding.source_location.clone(),
                    is_moved: false,
                    borrow_state: BorrowState::None,
                    ffi_symbol: None,
                };
                self.symbol_table.add_symbol(exception_symbol)?;
            }

            // Remove caught exception from current exceptions while analyzing handler
            let saved_handler_exceptions = self.current_exceptions.clone();
            self.current_exceptions.retain(|t| t != &exception_type);

            // Analyze handler block
            self.analyze_block(&catch.handler_block)?;

            // Restore exceptions and exit scope
            self.current_exceptions = saved_handler_exceptions;
            self.symbol_table.exit_scope()?;
        }

        // Analyze finally block if present
        if let Some(finally) = finally_block {
            let was_in_finally = self.in_finally_block;
            self.in_finally_block = true;

            // Finally blocks shouldn't throw new exceptions
            let saved_finally_exceptions = self.current_exceptions.clone();
            self.analyze_block(finally)?;

            // Validate no new exceptions were introduced in finally
            if self.current_exceptions.len() > saved_finally_exceptions.len() {
                return Err(SemanticError::InvalidOperation {
                    operation: "throw in finally block".to_string(),
                    reason: "Finally blocks should not throw new exceptions".to_string(),
                    location: finally.source_location.clone(),
                });
            }

            self.in_finally_block = was_in_finally;
        }

        // Restore exception context
        self.current_exceptions = saved_exceptions;
        Ok(())
    }

    /// Analyze a throw statement
    fn analyze_throw_statement(
        &mut self,
        exception: &Expression,
        source_location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // Validate we're not in a finally block
        if self.in_finally_block {
            return Err(SemanticError::InvalidOperation {
                operation: "throw in finally block".to_string(),
                reason: "Finally blocks should not throw exceptions except for cleanup".to_string(),
                location: source_location.clone(),
            });
        }

        // Analyze exception expression and get its type
        let exception_type = self.analyze_expression(exception)?;

        // Validate that the exception type is throwable
        if !self.is_throwable_type(&exception_type) {
            return Err(SemanticError::InvalidType {
                type_name: format!("{:?}", exception_type),
                reason: "type is not throwable (must implement Exception trait)".to_string(),
                location: source_location.clone(),
            });
        }

        // Add to current exceptions that can propagate
        if !self.current_exceptions.contains(&exception_type) {
            self.current_exceptions.push(exception_type.clone());
        }

        Ok(())
    }

    /// Analyze resource scope statement
    fn analyze_resource_scope(
        &mut self,
        scope: &crate::ast::resource::ResourceScope,
    ) -> Result<(), SemanticError> {
        use crate::resource::ResourceAnalyzer;

        // Create a resource analyzer for this scope
        let mut resource_analyzer = ResourceAnalyzer::new();

        // Analyze the resource scope
        resource_analyzer.analyze_resource_scope(scope)?;

        // Check for immediate issues
        let results = resource_analyzer.get_results();

        // Report any leaks detected during analysis
        if let Some(leak) = results.leaks.first() {
            return Err(SemanticError::ResourceLeak {
                resource_type: leak.resource_type.clone(),
                binding: leak.binding.clone(),
                location: leak.acquisition_location.clone(),
            });
        }

        // Report double releases
        if let Some(double_release) = results.double_releases.first() {
            return Err(SemanticError::InvalidOperation {
                operation: "double release".to_string(),
                reason: format!("Resource '{}' released twice", double_release.binding),
                location: double_release.second_release.clone(),
            });
        }

        // Report use after release
        if let Some(use_after_release) = results.use_after_release.first() {
            return Err(SemanticError::InvalidOperation {
                operation: "use after release".to_string(),
                reason: format!(
                    "Resource '{}' used after release",
                    use_after_release.binding
                ),
                location: use_after_release.use_location.clone(),
            });
        }

        // Analyze the body with resource bindings in scope
        self.symbol_table
            .enter_scope(crate::symbols::ScopeKind::Block);

        // Add resource bindings to symbol table
        for resource in &scope.resources {
            let resource_type = self.resolve_resource_type(&resource.resource_type)?;
            let symbol = crate::symbols::Symbol {
                name: resource.binding.name.clone(),
                symbol_type: resource_type,
                kind: crate::symbols::SymbolKind::Variable,
                is_mutable: false,
                is_initialized: true,
                declaration_location: resource.binding.source_location.clone(),
                is_moved: false,
                borrow_state: crate::symbols::BorrowState::None,
                ffi_symbol: None,
            };
            self.symbol_table.add_symbol(symbol)?;
        }

        // Analyze the body
        self.analyze_block(&scope.body)?;

        let _ = self.symbol_table.exit_scope();

        Ok(())
    }

    /// Resolve resource type to actual Type
    fn resolve_resource_type(&self, resource_type_name: &str) -> Result<Type, SemanticError> {
        // Map common resource types to their actual types
        match resource_type_name {
            "file_handle" => Ok(Type::primitive(PrimitiveType::UIntPtrT)),
            "memory_buffer" => Ok(Type::pointer(Type::primitive(PrimitiveType::Integer), true)),
            "tcp_socket" | "udp_socket" => Ok(Type::primitive(PrimitiveType::Integer32)),
            "mutex" | "semaphore" => Ok(Type::primitive(PrimitiveType::UIntPtrT)),
            _ => {
                // Try to resolve as a user-defined type
                if let Some(symbol) = self.symbol_table.lookup_symbol(resource_type_name) {
                    Ok(symbol.symbol_type.clone())
                } else {
                    // Default to opaque pointer for unknown resource types
                    Ok(Type::pointer(Type::primitive(PrimitiveType::Void), false))
                }
            }
        }
    }

    /// Check if a type is throwable (implements Exception trait or is built-in exception)
    fn is_throwable_type(&self, ty: &Type) -> bool {
        match ty {
            // Built-in exception types are always throwable
            Type::Named { name, .. } if name.ends_with("Error") || name.ends_with("Exception") => {
                true
            }

            // String can be thrown as a simple exception
            Type::Primitive(crate::ast::PrimitiveType::String) => true,

            // TODO: Check if type implements Exception trait
            // For now, accept most structured types as potentially throwable
            Type::Named { .. } => true,

            // Primitive types (except string) are not throwable
            Type::Primitive(_) => false,

            _ => false,
        }
    }

    /// Analyze an external function declaration
    fn analyze_external_function(
        &mut self,
        ext_func: &ExternalFunction,
    ) -> Result<(), SemanticError> {
        // Use FFI analyzer to validate the external function
        self.ffi_analyzer.analyze_external_function(ext_func)?;

        // Create function type for symbol table
        let mut param_types = Vec::new();
        for param in &ext_func.parameters {
            let param_type = self
                .type_checker
                .borrow()
                .ast_type_to_type(&param.param_type)?;
            param_types.push(param_type);
        }

        let return_type = self
            .type_checker
            .borrow()
            .ast_type_to_type(&ext_func.return_type)?;

        let func_type = if ext_func.variadic {
            Type::variadic_function(param_types.clone(), return_type.clone())
        } else {
            Type::function(param_types.clone(), return_type.clone())
        };

        // Check if external function already exists
        if let Some(existing_symbol) = self.symbol_table.lookup_symbol(&ext_func.name.name) {
            // For external functions, allow redeclaration if the signatures match
            if existing_symbol.kind == SymbolKind::Function {
                // Check if types match
                if !self
                    .type_checker
                    .borrow()
                    .types_compatible(&existing_symbol.symbol_type, &func_type)
                {
                    return Err(SemanticError::TypeMismatch {
                        expected: existing_symbol.symbol_type.to_string(),
                        found: func_type.to_string(),
                        location: ext_func.source_location.clone(),
                    });
                }
                // Types match, skip adding duplicate
                eprintln!("INFO: External function '{}' already declared with same signature, skipping duplicate", ext_func.name.name);
                return Ok(());
            } else {
                // Symbol exists but is not a function
                return Err(SemanticError::DuplicateDefinition {
                    symbol: ext_func.name.name.clone(),
                    location: ext_func.source_location.clone(),
                    previous_location: existing_symbol.declaration_location.clone(),
                });
            }
        }

        // Add external function to symbol table
        let func_symbol = Symbol {
            name: ext_func.name.name.clone(),
            symbol_type: Type::Function {
                parameter_types: param_types,
                return_type: Box::new(return_type),
                is_variadic: ext_func.variadic,
            },
            kind: SymbolKind::Function, // External functions are treated as regular functions
            is_mutable: false,
            is_initialized: true,
            declaration_location: ext_func.source_location.clone(),
            is_moved: false,
            borrow_state: BorrowState::None,
            ffi_symbol: ext_func.symbol.clone(),
        };

        self.symbol_table.add_symbol(func_symbol)?;
        self.stats.external_functions_analyzed += 1;

        Ok(())
    }

    /// Get FFI analyzer for generating bindings
    pub fn get_ffi_analyzer(&self) -> &FFIAnalyzer {
        &self.ffi_analyzer
    }

    /// Get analysis results
    pub fn get_statistics(&self) -> &AnalysisStats {
        &self.stats
    }

    /// Get the symbol table
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Get capture analysis results
    pub fn get_captures(&self) -> &HashMap<SourceLocation, std::collections::HashSet<String>> {
        &self.captures
    }

    /// Get trait dispatch table for method resolution
    pub fn get_trait_dispatch_table(&self) -> &HashMap<TraitMethodKey, TraitMethodDispatch> {
        &self.trait_dispatch_table
    }

    /// Get collected errors
    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }

    /// Check if analysis found any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get object files from ABI-loaded dependencies
    /// Returns paths to .o files that need to be linked for pre-compiled modules
    pub fn get_dependency_object_files(&self) -> Vec<std::path::PathBuf> {
        self.analyzed_modules
            .values()
            .filter_map(|m| m.object_file.clone())
            .collect()
    }

    /// Analyze a pattern and set up bindings
    fn analyze_pattern(
        &mut self,
        pattern: &Pattern,
        expected_type: &Type,
    ) -> Result<(), SemanticError> {
        match pattern {
            Pattern::EnumVariant {
                enum_name: _,
                variant_name,
                bindings,
                nested_pattern,
                source_location,
            } => {
                // Check that the pattern matches the expected enum type
                if let Type::Named {
                    name: enum_type_name,
                    ..
                } = expected_type
                {
                    // Find the enum definition
                    let enum_def = self
                        .type_checker
                        .borrow()
                        .lookup_type_definition(enum_type_name)
                        .cloned()
                        .ok_or_else(|| SemanticError::UndefinedSymbol {
                            symbol: enum_type_name.clone(),
                            location: source_location.clone(),
                        })?;

                    if let crate::types::TypeDefinition::Enum { variants, .. } = enum_def {
                        // Find the matching variant
                        let variant = variants
                            .iter()
                            .find(|v| v.name == variant_name.name)
                            .ok_or_else(|| SemanticError::UndefinedSymbol {
                                symbol: format!("variant '{}'", variant_name.name),
                                location: source_location.clone(),
                            })?;

                        // Handle nested pattern
                        if let Some(ref nested_pat) = nested_pattern {
                            if let Some(associated_type) = variant.associated_types.first() {
                                // Recursively analyze the nested pattern with the associated type
                                self.analyze_pattern(nested_pat, associated_type)?;
                            } else {
                                return Err(SemanticError::InvalidOperation {
                                    operation: "nested pattern matching".to_string(),
                                    reason: format!(
                                        "variant '{}' has no associated data",
                                        variant_name.name
                                    ),
                                    location: source_location.clone(),
                                });
                            }
                        }

                        // If there are bindings (without nested pattern), add them to the symbol table
                        if !bindings.is_empty() && nested_pattern.is_none() {
                            if bindings.len() != variant.associated_types.len() {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    function: variant_name.name.clone(), // Using function for variant name
                                    expected: variant.associated_types.len(),
                                    found: bindings.len(),
                                    location: source_location.clone(),
                                });
                            }

                            for (binding_id, associated_type) in
                                bindings.iter().zip(variant.associated_types.iter())
                            {
                                self.symbol_table.add_symbol(Symbol {
                                    name: binding_id.name.clone(),
                                    symbol_type: associated_type.clone(),
                                    kind: SymbolKind::Variable,
                                    is_mutable: false,
                                    is_initialized: true,
                                    declaration_location: binding_id.source_location.clone(),
                                    is_moved: false,
                                    borrow_state: BorrowState::None,
                                    ffi_symbol: None,
                                })?;
                            }
                        }
                    } else {
                        return Err(SemanticError::TypeMismatch {
                            expected: "enum type".to_string(),
                            found: enum_type_name.clone(),
                            location: source_location.clone(),
                        });
                    }
                } else {
                    return Err(SemanticError::TypeMismatch {
                        expected: "enum type".to_string(),
                        found: expected_type.to_string(),
                        location: source_location.clone(),
                    });
                }
            }

            Pattern::Wildcard { binding, .. } => {
                // Wildcard matches anything, bind the entire value if requested
                if let Some(binding_id) = binding {
                    self.symbol_table.add_symbol(Symbol {
                        name: binding_id.name.clone(),
                        symbol_type: expected_type.clone(),
                        kind: SymbolKind::Variable,
                        is_mutable: false,
                        is_initialized: true,
                        declaration_location: binding_id.source_location.clone(),
                        is_moved: false,
                        borrow_state: BorrowState::None,
                        ffi_symbol: None,
                    })?;
                }
            }

            Pattern::Literal { .. } => {
                // Literal patterns don't create bindings
            }

            Pattern::Struct {
                struct_name,
                fields,
                source_location,
            } => {
                // Check that expected_type is a struct with the given name
                if let Type::Named { name, .. } = expected_type {
                    if name != &struct_name.name {
                        return Err(SemanticError::TypeMismatch {
                            expected: struct_name.name.clone(),
                            found: name.clone(),
                            location: source_location.clone(),
                        });
                    }

                    // Look up struct definition
                    let type_def = self
                        .type_checker
                        .borrow()
                        .lookup_type_definition(name)
                        .cloned()
                        .ok_or_else(|| SemanticError::UndefinedSymbol {
                            symbol: name.clone(),
                            location: source_location.clone(),
                        })?;

                    if let crate::types::TypeDefinition::Struct {
                        fields: def_fields, ..
                    } = type_def
                    {
                        // Check fields
                        for (field_name, field_pattern) in fields {
                            // Find field type
                            let field_type = def_fields
                                .iter()
                                .find(|(fname, _)| fname == &field_name.name)
                                .map(|(_, ftype)| ftype)
                                .ok_or_else(|| SemanticError::UnknownField {
                                    struct_name: name.clone(),
                                    field_name: field_name.name.clone(),
                                    location: source_location.clone(),
                                })?;

                            // Recursively analyze
                            self.analyze_pattern(field_pattern, field_type)?;
                        }
                    } else {
                        return Err(SemanticError::TypeMismatch {
                            expected: "Struct".to_string(),
                            found: format!("{:?}", type_def),
                            location: source_location.clone(),
                        });
                    }
                } else {
                    return Err(SemanticError::TypeMismatch {
                        expected: struct_name.name.clone(),
                        found: expected_type.to_string(),
                        location: source_location.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check if a set of match patterns is exhaustive for the given enum type
    fn check_match_exhaustiveness(
        &self,
        patterns: &[&Pattern],
        enum_type: &Type,
        location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // Extract the enum type name
        let enum_type_name = match enum_type {
            Type::Named { name, .. } => name,
            _ => return Ok(()), // Not an enum, skip exhaustiveness check
        };

        // Get the enum definition
        let enum_def = self
            .type_checker
            .borrow()
            .lookup_type_definition(enum_type_name)
            .cloned()
            .ok_or_else(|| SemanticError::UndefinedSymbol {
                symbol: enum_type_name.clone(),
                location: location.clone(),
            })?;

        if let crate::types::TypeDefinition::Enum { variants, .. } = enum_def {
            // Check if there's a wildcard pattern
            let has_wildcard = patterns
                .iter()
                .any(|p| matches!(p, Pattern::Wildcard { .. }));

            if has_wildcard {
                // Wildcard makes the match exhaustive
                return Ok(());
            }

            // Collect all covered variant names
            let mut covered_variants = std::collections::HashSet::new();

            for pattern in patterns {
                if let Pattern::EnumVariant { variant_name, .. } = pattern {
                    covered_variants.insert(variant_name.name.clone());
                }
            }

            // Check if all variants are covered
            let mut missing_variants = Vec::new();
            for variant in &variants {
                if !covered_variants.contains(&variant.name) {
                    missing_variants.push(variant.name.clone());
                }
            }

            if !missing_variants.is_empty() {
                return Err(SemanticError::InvalidOperation {
                    operation: "match expression".to_string(),
                    reason: format!(
                        "non-exhaustive patterns: missing variants {}",
                        missing_variants.join(", ")
                    ),
                    location: location.clone(),
                });
            }
        }

        Ok(())
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;

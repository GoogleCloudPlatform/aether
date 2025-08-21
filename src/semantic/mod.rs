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

pub mod metadata;
// #[cfg(test)]
// mod ownership_tests;

use crate::ast::*;
use crate::contracts::{ContractValidator, ContractContext};
use crate::ffi::FFIAnalyzer;
use crate::memory::MemoryAnalyzer;
use crate::module_loader::{ModuleLoader, LoadedModule};
use crate::types::{Type, TypeChecker, OwnershipKind};
use crate::symbols::{Symbol, SymbolTable, SymbolKind, ScopeKind, BorrowState};
use crate::error::{SemanticError, SourceLocation};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

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
    
    /// Analyzed modules cache to prevent double-analysis
    analyzed_modules: HashMap<String, LoadedModule>,
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
            analyzed_modules: HashMap::new(),
        }
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
        self.current_module = Some(module.name.name.clone());
        self.symbol_table.set_current_module(self.current_module.clone());
        self.type_checker.borrow_mut().set_current_module(self.current_module.clone());
        
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
        
        // Second pass: Analyze function bodies
        for func_def in &module.function_definitions {
            self.analyze_function_body(func_def)?;
        }
        
        // Process exports (validate that exported symbols exist)
        for export in &module.exports {
            self.analyze_export(export)?;
        }
        
        // Exit module scope
        self.symbol_table.exit_scope()?;
        
        // Exit the root memory region
        self.memory_analyzer.exit_region()?;
        
        self.stats.modules_analyzed += 1;
        
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
        let loaded_module = self.module_loader.load_module(module_name)
            .map_err(|e| SemanticError::ImportError {
                module: module_name.clone(),
                reason: format!("Failed to load module: {}", e),
                location: import.source_location.clone(),
            })?;
        
        // Clone the module and dependencies to avoid borrow issues
        let module_to_analyze = loaded_module.module.clone();
        let loaded_module_clone = loaded_module.clone();
        
        // Store current module context
        let prev_module = self.current_module.clone();
        
        // Analyze the imported module
        self.current_module = Some(module_name.clone());
        if let Err(e) = self.analyze_module(&module_to_analyze) {
            self.current_module = prev_module;
            return Err(SemanticError::ImportError {
                module: module_name.clone(),
                reason: format!("Failed to analyze module: {}", e),
                location: import.source_location.clone(),
            });
        }
        
        // Restore module context
        self.current_module = prev_module;
        
        // Cache the analyzed module
        self.analyzed_modules.insert(module_name.clone(), loaded_module_clone);
        
        // Add imported module to current scope
        self.add_imported_module_to_scope(module_name, alias, &import.source_location)?;
        
        Ok(())
    }
    
    /// Add imported module symbols to current scope
    fn add_imported_module_to_scope(&mut self, module_name: &str, alias: Option<&String>, location: &SourceLocation) -> Result<(), SemanticError> {
        // Get the loaded module
        let loaded_module = self.analyzed_modules.get(module_name)
            .ok_or_else(|| SemanticError::Internal {
                message: format!("Module {} not found in analyzed modules cache", module_name),
            })?;
        
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
                    
                    // Look up the function type from the module's symbol table
                    // For now, we'll add a placeholder - full implementation would need
                    // to maintain module-specific symbol tables
                    let symbol = Symbol::new(
                        qualified_name,
                        Type::Function {
                            parameter_types: vec![],
                            return_type: Box::new(Type::Primitive(crate::ast::PrimitiveType::Void)),
                        },
                        SymbolKind::Function,
                        false,
                        true,
                        location.clone(),
                    );
                    
                    self.symbol_table.add_symbol(symbol)?;
                }
                ExportStatement::Type { name, .. } => {
                    // Add exported type to type system
                    let qualified_name = if let Some(alias_name) = alias {
                        format!("{}.{}", alias_name, name.name)
                    } else {
                        format!("{}.{}", module_name, name.name)
                    };
                    
                    // For now, add as a named type
                    let symbol = Symbol::new(
                        qualified_name.clone(),
                        Type::Named { 
                            name: qualified_name, 
                            module: Some(module_name.to_string()) 
                        },
                        SymbolKind::Type,
                        false,
                        true,
                        location.clone(),
                    );
                    
                    self.symbol_table.add_symbol(symbol)?;
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
                        qualified_name,
                        Type::Error, // Use Error type as placeholder for unknown constant type
                        SymbolKind::Constant,
                        false,
                        true,
                        location.clone(),
                    );
                    
                    self.symbol_table.add_symbol(symbol)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Analyze a type definition
    fn analyze_type_definition(&mut self, type_def: &crate::ast::TypeDefinition) -> Result<(), SemanticError> {
        match type_def {
            crate::ast::TypeDefinition::Structured { name, fields, source_location, .. } => {
                let mut field_types = Vec::new();
                
                // Analyze each field (preserving declaration order)
                for field in fields {
                    let field_type = self.type_checker.borrow().ast_type_to_type(&field.field_type)?;
                    field_types.push((field.name.name.clone(), field_type));
                }
                
                // Add the type definition
                let definition = crate::types::TypeDefinition::Struct {
                    fields: field_types.clone(),
                    source_location: source_location.clone(),
                };
                
                eprintln!("Semantic: Adding struct type '{}' to symbol table and type checker", name.name);
                self.symbol_table.add_type_definition(name.name.clone(), definition.clone())?;
                self.type_checker.borrow_mut().add_type_definition(name.name.clone(), definition);
            }
            
            crate::ast::TypeDefinition::Enumeration { name, variants, source_location, .. } => {
                // Convert AST variants to type system variants
                let mut variant_infos = Vec::new();
                for (idx, variant) in variants.iter().enumerate() {
                    let associated_type = if let Some(type_spec) = &variant.associated_type {
                        Some(self.type_checker.borrow().ast_type_to_type(type_spec)?)
                    } else {
                        None
                    };
                    
                    variant_infos.push(crate::types::EnumVariantInfo {
                        name: variant.name.name.clone(),
                        associated_type,
                        discriminant: idx, // Variants get indices based on declaration order
                    });
                }
                
                let definition = crate::types::TypeDefinition::Enum {
                    variants: variant_infos.clone(),
                    source_location: source_location.clone(),
                };
                
                self.symbol_table.add_type_definition(name.name.clone(), definition.clone())?;
                self.type_checker.borrow_mut().add_type_definition(name.name.clone(), definition);
            }
            
            crate::ast::TypeDefinition::Alias { new_name, original_type, source_location, .. } => {
                let target_type = self.type_checker.borrow().ast_type_to_type(original_type)?;
                
                let definition = crate::types::TypeDefinition::Alias {
                    target_type,
                    source_location: source_location.clone(),
                };
                
                self.symbol_table.add_type_definition(new_name.name.clone(), definition)?;
            }
        }
        
        self.stats.types_defined += 1;
        Ok(())
    }
    
    /// Analyze a constant declaration
    fn analyze_constant_declaration(&mut self, const_decl: &ConstantDeclaration) -> Result<(), SemanticError> {
        // Get the declared type
        let declared_type = self.type_checker.borrow().ast_type_to_type(&const_decl.type_spec)?;
        
        // Analyze the value expression
        let value_type = self.analyze_expression(&const_decl.value)?;
        
        // Check type compatibility
        if !self.type_checker.borrow().types_compatible(&declared_type, &value_type) {
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
        };
        
        self.symbol_table.add_symbol(symbol)?;
        self.stats.variables_declared += 1;
        
        Ok(())
    }
    
    /// Add function signature to symbol table (first pass)
    fn add_function_signature(&mut self, func_def: &Function) -> Result<(), SemanticError> {
        // Get the return type
        let return_type = self.type_checker.borrow().ast_type_to_type(&func_def.return_type)?;
        
        // Analyze parameters
        let mut param_types = Vec::new();
        for param in &func_def.parameters {
            let param_type = self.type_checker.borrow().ast_type_to_type(&param.param_type)?;
            param_types.push(param_type);
        }
        
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
        };
        
        self.symbol_table.add_symbol(func_symbol)?;
        Ok(())
    }

    /// Analyze function body (second pass)
    fn analyze_function_body(&mut self, func_def: &Function) -> Result<(), SemanticError> {
        
        // Enter function scope
        self.symbol_table.enter_scope(ScopeKind::Function);
        
        // Add parameters to function scope
        for param in &func_def.parameters {
            let param_type = self.type_checker.borrow().ast_type_to_type(&param.param_type)?;
            let param_symbol = Symbol {
                name: param.name.name.clone(),
                symbol_type: param_type,
                kind: SymbolKind::Parameter,
                is_mutable: true, // Parameters are typically mutable in their scope
                is_initialized: true,
                declaration_location: param.source_location.clone(),
            is_moved: false,
            borrow_state: BorrowState::None,
            };
            
            self.symbol_table.add_symbol(param_symbol)?;
        }
        
        // Analyze memory allocation strategy for this function
        let memory_info = self.memory_analyzer.analyze_function(func_def)?;
        
        // TODO: Store memory_info for later use in code generation
        // For now, we'll just log it in debug mode
        #[cfg(debug_assertions)]
        {
            eprintln!("Memory analysis for function '{}': {:?}", func_def.name.name, memory_info);
        }
        
        // Analyze function body
        self.analyze_block(&func_def.body)?;
        
        // Validate function metadata and contracts
        self.validate_function_contracts(func_def)?;
        
        // Exit function scope
        self.symbol_table.exit_scope()?;
        self.stats.functions_analyzed += 1;
        
        Ok(())
    }
    
    /// Validate function contracts and metadata
    fn validate_function_contracts(&mut self, func_def: &Function) -> Result<(), SemanticError> {
        // Create contract context
        let mut parameter_types = HashMap::new();
        for param in &func_def.parameters {
            let param_type = self.type_checker.borrow().ast_type_to_type(&param.param_type)?;
            parameter_types.insert(param.name.name.clone(), param_type);
        }
        
        let return_type = self.type_checker.borrow().ast_type_to_type(&func_def.return_type)?;
        
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
                    eprintln!("Contract warning in function '{}': {}", func_def.name.name, warning);
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
            ExportStatement::Function { name, source_location } |
            ExportStatement::Type { name, source_location } |
            ExportStatement::Constant { name, source_location } => {
                // Check that the exported symbol exists
                if self.symbol_table.lookup_symbol(&name.name).is_none() {
                    return Err(SemanticError::UndefinedSymbol {
                        symbol: name.name.clone(),
                        location: source_location.clone(),
                    });
                }
            }
        }
        
        Ok(())
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
                eprintln!("Semantic: About to look up type in variable declaration");
                let declared_type = self.type_checker.borrow().ast_type_to_type(type_spec)?;
                let is_mutable = matches!(mutability, Mutability::Mutable);
                let mut is_initialized = false;
                
                // If there's an initial value, analyze it and check type compatibility
                if let Some(init_expr) = initial_value {
                    let init_type = self.analyze_expression(init_expr)?;
                    
                    if !self.type_checker.borrow().types_compatible(&declared_type, &init_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: declared_type.to_string(),
                            found: init_type.to_string(),
                            location: source_location.clone(),
                        });
                    }
                    
                    is_initialized = true;
                }
                
                // Add variable to symbol table
                let symbol = Symbol::new(
                    name.name.clone(),
                    declared_type,
                    SymbolKind::Variable,
                    is_mutable,
                    is_initialized,
                    source_location.clone(),
                );
                
                self.symbol_table.add_symbol(symbol)?;
                self.stats.variables_declared += 1;
            }
            
            Statement::Assignment { target, value, source_location } => {
                let value_type = self.analyze_expression(value)?;
                
                match target {
                    AssignmentTarget::Variable { name } => {
                        // Check that variable exists and is mutable
                        let symbol = self.symbol_table.lookup_symbol(&name.name)
                            .ok_or_else(|| SemanticError::UndefinedSymbol {
                                symbol: name.name.clone(),
                                location: source_location.clone(),
                            })?;
                        
                        if !symbol.is_mutable {
                            return Err(SemanticError::AssignToImmutable {
                                variable: name.name.clone(),
                                location: source_location.clone(),
                            });
                        }
                        
                        // Check type compatibility
                        if !self.type_checker.borrow().types_compatible(&symbol.symbol_type, &value_type) {
                            return Err(SemanticError::TypeMismatch {
                                expected: symbol.symbol_type.to_string(),
                                found: value_type.to_string(),
                                location: source_location.clone(),
                            });
                        }
                        
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
            
            Statement::Return { value, .. } => {
                if let Some(return_expr) = value {
                    self.analyze_expression(return_expr)?;
                    // TODO: Check that return type matches function signature
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
                            if matches!(symbol.borrow_state, BorrowState::Borrowed(_) | BorrowState::BorrowedMut) {
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
            
            Statement::If { condition, then_block, else_ifs, else_block, .. } => {
                self.analyze_if_statement(condition, then_block, else_ifs, else_block)?;
            }
            
            Statement::WhileLoop { condition, body, invariant, .. } => {
                self.analyze_while_loop(condition, body, invariant)?;
            }
            
            Statement::ForEachLoop { collection, element_binding, element_type, body, .. } => {
                self.analyze_for_each_loop(collection, element_binding, element_type, body)?;
            }
            
            Statement::FixedIterationLoop { counter, from_value, to_value, step_value, body, .. } => {
                self.analyze_fixed_iteration_loop(counter, from_value, to_value, step_value, body)?;
            }
            
            Statement::Break { target_label, source_location } => {
                self.analyze_break_statement(target_label, source_location)?;
            }
            
            Statement::Continue { target_label, source_location } => {
                self.analyze_continue_statement(target_label, source_location)?;
            }
            
            Statement::TryBlock { protected_block, catch_clauses, finally_block, .. } => {
                self.analyze_try_block(protected_block, catch_clauses, finally_block)?;
            }
            
            Statement::Throw { exception, source_location } => {
                self.analyze_throw_statement(exception, source_location)?;
            }
            
            Statement::ResourceScope { scope, .. } => {
                self.analyze_resource_scope(scope)?;
            }
            
            Statement::Expression { expr, .. } => {
                // For expression statements, just analyze the expression
                self.analyze_expression(expr)?;
            }
        }
        
        Ok(())
    }
    
    /// Analyze an expression and return its type
    fn analyze_expression(&mut self, expression: &Expression) -> Result<Type, SemanticError> {
        match expression {
            Expression::IntegerLiteral { .. } => {
                Ok(Type::primitive(PrimitiveType::Integer))
            }
            
            Expression::FloatLiteral { .. } => {
                Ok(Type::primitive(PrimitiveType::Float))
            }
            
            Expression::StringLiteral { .. } => {
                Ok(Type::primitive(PrimitiveType::String))
            }
            
            Expression::CharacterLiteral { .. } => {
                Ok(Type::primitive(PrimitiveType::Char))
            }
            
            Expression::BooleanLiteral { .. } => {
                Ok(Type::primitive(PrimitiveType::Boolean))
            }
            
            Expression::NullLiteral { .. } => {
                // Null can be any pointer type - return a generic pointer for now
                Ok(Type::pointer(Type::primitive(PrimitiveType::Void), false))
            }
            
            Expression::Variable { name, source_location } => {
                let symbol = self.symbol_table.lookup_symbol(&name.name)
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
            
            Expression::Add { left, right, source_location } |
            Expression::Subtract { left, right, source_location } |
            Expression::Multiply { left, right, source_location } |
            Expression::Divide { left, right, source_location } => {
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
            
            Expression::FunctionCall { call, source_location } => {
                self.analyze_function_call_expression(call, source_location)
            }
            
            Expression::StringConcat { operands, source_location } => {
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
            
            Expression::StringLength { string, source_location } => {
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
            Expression::StringCharAt { string, index, source_location } => {
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
            
            Expression::Substring { string, start_index, length, source_location } => {
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
            
            Expression::StringEquals { left, right, source_location } => {
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
            
            Expression::StringContains { haystack, needle, source_location } => {
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
            
            Expression::ArrayLiteral { element_type, elements, source_location } => {
                // Convert AST type to semantic type
                let expected_element_type = self.type_checker.borrow().ast_type_to_type(element_type)?;
                
                // Check all elements match the declared type
                for element in elements {
                    let element_type = self.analyze_expression(element)?;
                    if !self.type_checker.borrow().types_compatible(&expected_element_type, &element_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_element_type.to_string(),
                            found: element_type.to_string(),
                            location: source_location.clone(),
                        });
                    }
                }
                
                Ok(Type::array(expected_element_type, Some(elements.len())))
            }
            
            Expression::ArrayAccess { array, index, source_location: _ } => {
                let array_type = self.analyze_expression(array)?;
                
                // Check that it's an array
                match array_type {
                    Type::Array { element_type, .. } => {
                        // Index must be integer
                        let index_type = self.analyze_expression(index)?;
                        if !matches!(index_type, Type::Primitive(PrimitiveType::Integer)) {
                            return Err(SemanticError::TypeMismatch {
                                expected: "Integer".to_string(),
                                found: index_type.to_string(),
                                location: SourceLocation::unknown(),
                            });
                        }
                        
                        Ok((*element_type).clone())
                    }
                    _ => {
                        Err(SemanticError::TypeMismatch {
                            expected: "Array".to_string(),
                            found: array_type.to_string(),
                            location: SourceLocation::unknown(),
                        })
                    }
                }
            }
            
            Expression::ArrayLength { array, source_location } => {
                let array_type = self.analyze_expression(array)?;
                
                // Check that it's an array
                match array_type {
                    Type::Array { .. } => Ok(Type::primitive(PrimitiveType::Integer)),
                    _ => {
                        Err(SemanticError::TypeMismatch {
                            expected: "Array".to_string(),
                            found: array_type.to_string(),
                            location: source_location.clone(),
                        })
                    }
                }
            }
            
            Expression::StructConstruct { type_name, field_values, source_location } => {
                // Look up the struct type
                eprintln!("Semantic: Looking up struct type '{}'", type_name.name);
                
                // Clone the fields to avoid borrowing issues
                let fields_clone = {
                    let type_def = self.symbol_table.lookup_type_definition(&type_name.name)
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
                    if !field_values.iter().any(|fv| fv.field_name.name == *field_name) {
                        return Err(SemanticError::MissingField {
                            struct_name: type_name.name.clone(),
                            field_name: field_name.clone(),
                            location: source_location.clone(),
                        });
                    }
                }
                
                // Check field types
                for field_value in field_values {
                    let expected_type = fields_clone.iter()
                        .find(|(name, _)| name == &field_value.field_name.name)
                        .map(|(_, ty)| ty)
                        .ok_or_else(|| SemanticError::UnknownField {
                            struct_name: type_name.name.clone(),
                            field_name: field_value.field_name.name.clone(),
                            location: field_value.source_location.clone(),
                        })?;
                    
                    let value_type = self.analyze_expression(&field_value.value)?;
                    if !self.type_checker.borrow().types_compatible(expected_type, &value_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.to_string(),
                            found: value_type.to_string(),
                            location: field_value.source_location.clone(),
                        });
                    }
                }
                
                // Return the struct type
                Ok(Type::named(type_name.name.clone(), self.current_module.clone()))
            }
            
            Expression::FieldAccess { instance, field_name, source_location } => {
                // Get the type of the instance
                let instance_type = self.analyze_expression(instance)?;
                
                // Check if it's a named type (struct)
                match &instance_type {
                    Type::Named { name, module } => {
                        // Look up the struct definition
                        let full_name = if let Some(mod_name) = module {
                            format!("{}::{}", mod_name, name)
                        } else {
                            name.clone()
                        };
                        
                        let type_def = self.symbol_table.lookup_type_definition(&name)
                            .ok_or_else(|| SemanticError::UndefinedSymbol {
                                symbol: full_name.clone(),
                                location: source_location.clone(),
                            })?;
                        
                        // Check that it's a struct and get the field type
                        if let crate::types::TypeDefinition::Struct { fields, .. } = type_def {
                            let field_type = fields.iter()
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
                    _ => {
                        Err(SemanticError::TypeMismatch {
                            expected: "struct type".to_string(),
                            found: instance_type.to_string(),
                            location: source_location.clone(),
                        })
                    }
                }
            }
            
            Expression::Equals { left, right, source_location } => {
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
            
            Expression::NotEquals { left, right, source_location } => {
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
            
            Expression::EnumVariant { enum_name, variant_name, value, source_location } => {
                eprintln!("Semantic: Analyzing enum variant construction: {}", variant_name.name);
                
                // For now, we need to find the enum type by looking through all types
                // In the future, we should improve this by having better variant lookup
                let module_name = self.current_module.clone().unwrap_or_default();
                let enum_type = self.type_checker.borrow().find_enum_type_by_variant(&variant_name.name, &module_name)
                    .ok_or_else(|| SemanticError::UndefinedSymbol {
                        symbol: format!("enum variant '{}'", variant_name.name),
                        location: source_location.clone(),
                    })?;
                
                // Check if the variant has an associated value
                let variant = enum_type.get_variant(&variant_name.name)
                    .ok_or_else(|| SemanticError::UndefinedSymbol {
                        symbol: format!("variant '{}' in enum", variant_name.name),
                        location: source_location.clone(),
                    })?;
                
                // Type check the associated value if present
                if let Some(expected_type) = &variant.associated_type {
                    if let Some(value_expr) = value {
                        let value_type = self.analyze_expression(value_expr)?;
                        self.type_checker.borrow().check_type_compatibility(expected_type, &value_type, source_location)?;
                    } else {
                        return Err(SemanticError::MissingEnumVariantValue {
                            variant: variant_name.name.clone(),
                            enum_name: enum_type.name.clone(),
                            location: source_location.clone(),
                        });
                    }
                } else if value.is_some() {
                    return Err(SemanticError::UnexpectedEnumVariantValue {
                        variant: variant_name.name.clone(),
                        enum_name: enum_type.name.clone(),
                        location: source_location.clone(),
                    });
                }
                
                // Return the enum type
                Ok(Type::Named {
                    name: enum_type.name.clone(),
                    module: self.current_module.clone(),
                })
            }
            
            Expression::Match { value, cases, source_location } => {
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
                        if !self.type_checker.borrow().are_types_equal(expected_type, &case_type) {
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
            
            Expression::TypeCast { value, target_type, failure_behavior: _, source_location } => {
                let value_type = self.analyze_expression(value)?;
                let target = self.type_checker.borrow().ast_type_to_type(target_type)?;
                
                // TODO: Check if the cast is valid
                // For now, we'll allow casts between primitive types
                match (&value_type, &target) {
                    (Type::Primitive(from), Type::Primitive(to)) => {
                        // Allow numeric to string conversions
                        if matches!(to, PrimitiveType::String) && (from.is_numeric() || matches!(from, PrimitiveType::Boolean)) {
                            Ok(target)
                        }
                        // Allow string to numeric conversions
                        else if matches!(from, PrimitiveType::String) && to.is_numeric() {
                            Ok(target)
                        }
                        // Allow numeric to numeric conversions
                        else if from.is_numeric() && to.is_numeric() {
                            Ok(target)
                        }
                        else {
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
                    })
                }
            }
            
            Expression::AddressOf { operand, source_location } => {
                let operand_type = self.analyze_expression(operand)?;
                // Create a pointer type to the operand type
                Ok(Type::pointer(operand_type, false))
            }
            
            Expression::Dereference { pointer, source_location } => {
                let pointer_type = self.analyze_expression(pointer)?;
                // Check that it's a pointer type
                match pointer_type {
                    Type::Pointer { target_type, .. } => {
                        Ok((*target_type).clone())
                    }
                    _ => {
                        Err(SemanticError::TypeMismatch {
                            expected: "pointer type".to_string(),
                            found: pointer_type.to_string(),
                            location: source_location.clone(),
                        })
                    }
                }
            }
            
            Expression::PointerArithmetic { pointer, offset, operation: _, source_location } => {
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
                    _ => {
                        Err(SemanticError::TypeMismatch {
                            expected: "pointer type".to_string(),
                            found: pointer_type.to_string(),
                            location: source_location.clone(),
                        })
                    }
                }
            }
            
            Expression::MapLiteral { key_type, value_type, entries, source_location } => {
                // Convert AST types to semantic types
                let key_sem_type = self.type_checker.borrow().ast_type_to_type(key_type)?;
                let value_sem_type = self.type_checker.borrow().ast_type_to_type(value_type)?;
                
                // Check all entries match the declared types
                for entry in entries {
                    let entry_key_type = self.analyze_expression(&entry.key)?;
                    let entry_value_type = self.analyze_expression(&entry.value)?;
                    
                    if !self.type_checker.borrow().types_compatible(&key_sem_type, &entry_key_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: key_sem_type.to_string(),
                            found: entry_key_type.to_string(),
                            location: entry.source_location.clone(),
                        });
                    }
                    
                    if !self.type_checker.borrow().types_compatible(&value_sem_type, &entry_value_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: value_sem_type.to_string(),
                            found: entry_value_type.to_string(),
                            location: entry.source_location.clone(),
                        });
                    }
                }
                
                Ok(Type::map(key_sem_type, value_sem_type))
            }
            
            Expression::MapAccess { map, key, source_location } => {
                let map_type = self.analyze_expression(map)?;
                
                // Check that it's a map
                match map_type {
                    Type::Map { key_type, value_type } => {
                        // Check key type
                        let provided_key_type = self.analyze_expression(key)?;
                        if !self.type_checker.borrow().types_compatible(&*key_type, &provided_key_type) {
                            return Err(SemanticError::TypeMismatch {
                                expected: key_type.to_string(),
                                found: provided_key_type.to_string(),
                                location: source_location.clone(),
                            });
                        }
                        
                        Ok((*value_type).clone())
                    }
                    _ => {
                        Err(SemanticError::TypeMismatch {
                            expected: "Map".to_string(),
                            found: map_type.to_string(),
                            location: source_location.clone(),
                        })
                    }
                }
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
    fn analyze_assignment_target(&mut self, target: &AssignmentTarget) -> Result<Type, SemanticError> {
        match target {
            AssignmentTarget::Variable { name } => {
                let symbol = self.symbol_table.lookup_symbol(&name.name)
                    .ok_or_else(|| SemanticError::UndefinedSymbol {
                        symbol: name.name.clone(),
                        location: SourceLocation::unknown(), // TODO: Better location tracking
                    })?;
                
                Ok(symbol.symbol_type.clone())
            }
            
            AssignmentTarget::MapValue { map, key } => {
                let map_type = self.analyze_expression(map)?;
                
                // Check that it's a map
                match map_type {
                    Type::Map { key_type, value_type } => {
                        // Check key type
                        let provided_key_type = self.analyze_expression(key)?;
                        if !self.type_checker.borrow().types_compatible(&*key_type, &provided_key_type) {
                            return Err(SemanticError::TypeMismatch {
                                expected: key_type.to_string(),
                                found: provided_key_type.to_string(),
                                location: SourceLocation::unknown(),
                            });
                        }
                        
                        Ok((*value_type).clone())
                    }
                    _ => {
                        Err(SemanticError::TypeMismatch {
                            expected: "Map".to_string(),
                            found: map_type.to_string(),
                            location: SourceLocation::unknown(),
                        })
                    }
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
                    let symbol = self.symbol_table.lookup_symbol(&name.name)
                        .ok_or_else(|| SemanticError::UndefinedSymbol {
                            symbol: name.name.clone(),
                            location: SourceLocation::unknown(), // TODO: Better location tracking
                        })?;
                    
                    // Extract return type from function type
                    if let Type::Function { return_type, parameter_types } = &symbol.symbol_type {
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
                    let arg_type = self.analyze_expression(arg.value.as_ref())?;
                    
                    if let Some(param_type) = parameter_types.get(i) {
                        // Check ownership compatibility
                        let arg_ownership = arg_type.get_ownership();
                        let param_ownership = param_type.get_ownership();
                        
                        
                        match (arg_ownership, param_ownership) {
                            // Ownership transfer: owned to owned
                            (Some(OwnershipKind::Owned), Some(OwnershipKind::Owned)) => {
                                // Record move if argument is a variable
                                if let Expression::Variable { name: var_name, .. } = arg.value.as_ref() {
                                    if arg_type.requires_ownership() {
                                        self.symbol_table.mark_variable_moved(&var_name.name)?;
                                    }
                                }
                            }
                            // Borrowing: owned to borrowed
                            (Some(OwnershipKind::Owned), Some(OwnershipKind::Borrowed)) |
                            (None, Some(OwnershipKind::Borrowed)) => {
                                // Record immutable borrow if argument is a variable
                                if let Expression::Variable { name: var_name, .. } = arg.value.as_ref() {
                                    if arg_type.requires_ownership() {
                                        self.symbol_table.borrow_variable(&var_name.name)?;
                                    }
                                }
                            }
                            // Mutable borrowing: owned to mutable borrow
                            (Some(OwnershipKind::Owned), Some(OwnershipKind::MutableBorrow)) |
                            (None, Some(OwnershipKind::MutableBorrow)) => {
                                // Record mutable borrow if argument is a variable
                                if let Expression::Variable { name: var_name, .. } = arg.value.as_ref() {
                                    if arg_type.requires_ownership() {
                                        self.symbol_table.borrow_variable_mut(&var_name.name)?;
                                    }
                                }
                            }
                            // Default case: no ownership tracking needed
                            _ => {}
                        }
                        
                        // Check type compatibility
                        if !self.type_checker.borrow().types_compatible(param_type, &arg_type) {
                            return Err(SemanticError::TypeMismatch {
                                expected: param_type.to_string(),
                                found: arg_type.to_string(),
                                location: SourceLocation::unknown(),
                            });
                        }
                    }
                }
                
                // Handle variadic arguments ownership
                for (i, arg) in call.variadic_arguments.iter().enumerate() {
                    let arg_type = self.analyze_expression(arg.as_ref())?;
                    let param_index = call.arguments.len() + i;
                    
                    if let Some(param_type) = parameter_types.get(param_index) {
                        // Check ownership compatibility
                        let arg_ownership = arg_type.get_ownership();
                        let param_ownership = param_type.get_ownership();
                        
                        
                        match (arg_ownership, param_ownership) {
                            // Ownership transfer: owned to owned
                            (Some(OwnershipKind::Owned), Some(OwnershipKind::Owned)) => {
                                // Record move if argument is a variable
                                if let Expression::Variable { name: var_name, .. } = arg.as_ref() {
                                    if arg_type.requires_ownership() {
                                        self.symbol_table.mark_variable_moved(&var_name.name)?;
                                    }
                                }
                            }
                            // Borrowing: owned to borrowed
                            (Some(OwnershipKind::Owned), Some(OwnershipKind::Borrowed)) |
                            (None, Some(OwnershipKind::Borrowed)) => {
                                // Record immutable borrow if argument is a variable
                                if let Expression::Variable { name: var_name, .. } = arg.as_ref() {
                                    if arg_type.requires_ownership() {
                                        self.symbol_table.borrow_variable(&var_name.name)?;
                                    }
                                }
                            }
                            // Mutable borrowing
                            (Some(OwnershipKind::Owned), Some(OwnershipKind::MutableBorrow)) |
                            (None, Some(OwnershipKind::MutableBorrow)) => {
                                // Record mutable borrow if argument is a variable
                                if let Expression::Variable { name: var_name, .. } = arg.as_ref() {
                                    if arg_type.requires_ownership() {
                                        self.symbol_table.borrow_variable_mut(&var_name.name)?;
                                    }
                                }
                            }
                            // Other cases don't require special handling
                            _ => {}
                        }
                        
                        // Check type compatibility
                        if !self.type_checker.borrow().types_compatible(param_type, &arg_type) {
                            return Err(SemanticError::TypeMismatch {
                                expected: param_type.to_string(),
                                found: arg_type.to_string(),
                                location: SourceLocation::unknown(),
                            });
                        }
                    }
                }
                
                Ok(return_type)
            }
            
            // TODO: Handle qualified and external function references
            _ => Ok(Type::Error),
        }
    }
    
    /// Analyze a function call expression
    fn analyze_function_call_expression(&mut self, call: &FunctionCall, source_location: &SourceLocation) -> Result<Type, SemanticError> {
        self.analyze_function_call(call).map_err(|mut e| {
            // Update the source location if it's missing
            match &mut e {
                SemanticError::UndefinedSymbol { location, .. } |
                SemanticError::TypeMismatch { location, .. } |
                SemanticError::ArgumentCountMismatch { location, .. } => {
                    if location.file == "<unknown>" {
                        *location = source_location.clone();
                    }
                }
                _ => {}
            }
            e
        })
    }
    
    /// Analyze an if statement
    fn analyze_if_statement(&mut self, condition: &Expression, then_block: &Block, else_ifs: &[ElseIf], else_block: &Option<Block>) -> Result<(), SemanticError> {
        // Analyze condition - must be boolean
        let condition_type = self.analyze_expression(condition)?;
        if !matches!(condition_type, Type::Primitive(PrimitiveType::Boolean) | Type::Error) {
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
            if !matches!(else_if_condition_type, Type::Primitive(PrimitiveType::Boolean) | Type::Error) {
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
    fn analyze_while_loop(&mut self, condition: &Expression, body: &Block, invariant: &Option<String>) -> Result<(), SemanticError> {
        // Analyze condition - must be boolean
        let condition_type = self.analyze_expression(condition)?;
        if !matches!(condition_type, Type::Primitive(PrimitiveType::Boolean) | Type::Error) {
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
    fn analyze_for_each_loop(&mut self, collection: &Expression, element_binding: &Identifier, element_type: &TypeSpecifier, body: &Block) -> Result<(), SemanticError> {
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
        if !self.type_checker.borrow().types_compatible(&declared_element_type, &element_actual_type) {
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
        };
        self.symbol_table.add_symbol(element_symbol)?;
        
        // Analyze loop body
        self.analyze_block(body)?;
        
        // Exit loop scope
        self.symbol_table.exit_scope()?;
        
        Ok(())
    }
    
    /// Analyze a fixed iteration loop
    fn analyze_fixed_iteration_loop(&mut self, counter: &Identifier, from_value: &Expression, to_value: &Expression, step_value: &Option<Box<Expression>>, body: &Block) -> Result<(), SemanticError> {
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
        };
        self.symbol_table.add_symbol(counter_symbol)?;
        
        // Analyze loop body
        self.analyze_block(body)?;
        
        // Exit loop scope
        self.symbol_table.exit_scope()?;
        
        Ok(())
    }
    
    /// Analyze a break statement
    fn analyze_break_statement(&mut self, target_label: &Option<Identifier>, source_location: &SourceLocation) -> Result<(), SemanticError> {
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
    fn analyze_continue_statement(&mut self, target_label: &Option<Identifier>, source_location: &SourceLocation) -> Result<(), SemanticError> {
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
    fn analyze_try_block(&mut self, protected_block: &Block, catch_clauses: &[CatchClause], finally_block: &Option<Block>) -> Result<(), SemanticError> {
        // Track exception flow - save current state
        let saved_exceptions = self.current_exceptions.clone();
        let mut caught_exception_types = Vec::new();
        
        // Analyze protected block with exception tracking
        self.analyze_block(protected_block)?;
        
        // Validate catch clauses
        for catch in catch_clauses {
            // Validate exception type exists and is throwable
            let exception_type = self.type_checker.borrow().ast_type_to_type(&catch.exception_type)?;
            
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
    fn analyze_throw_statement(&mut self, exception: &Expression, source_location: &SourceLocation) -> Result<(), SemanticError> {
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
    fn analyze_resource_scope(&mut self, scope: &crate::ast::resource::ResourceScope) -> Result<(), SemanticError> {
        use crate::resource::ResourceAnalyzer;
        
        // Create a resource analyzer for this scope
        let mut resource_analyzer = ResourceAnalyzer::new();
        
        // Analyze the resource scope
        resource_analyzer.analyze_resource_scope(scope)?;
        
        // Check for immediate issues
        let results = resource_analyzer.get_results();
        
        // Report any leaks detected during analysis
        for leak in &results.leaks {
            return Err(SemanticError::ResourceLeak {
                resource_type: leak.resource_type.clone(),
                binding: leak.binding.clone(),
                location: leak.acquisition_location.clone(),
            });
        }
        
        // Report double releases
        for double_release in &results.double_releases {
            return Err(SemanticError::InvalidOperation {
                operation: "double release".to_string(),
                reason: format!("Resource '{}' released twice", double_release.binding),
                location: double_release.second_release.clone(),
            });
        }
        
        // Report use after release
        for use_after_release in &results.use_after_release {
            return Err(SemanticError::InvalidOperation {
                operation: "use after release".to_string(),
                reason: format!("Resource '{}' used after release", use_after_release.binding),
                location: use_after_release.use_location.clone(),
            });
        }
        
        // Analyze the body with resource bindings in scope
        self.symbol_table.enter_scope(crate::symbols::ScopeKind::Block);
        
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
            };
            self.symbol_table.add_symbol(symbol)?;
        }
        
        // Analyze the body
        self.analyze_block(&scope.body)?;
        
        self.symbol_table.exit_scope();
        
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
            Type::Named { name, .. } if name.ends_with("Error") || name.ends_with("Exception") => true,
            
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
    fn analyze_external_function(&mut self, ext_func: &ExternalFunction) -> Result<(), SemanticError> {
        // Use FFI analyzer to validate the external function
        self.ffi_analyzer.analyze_external_function(ext_func)?;
        
        // Create function type for symbol table
        let mut param_types = Vec::new();
        for param in &ext_func.parameters {
            let param_type = self.type_checker.borrow().ast_type_to_type(&param.param_type)?;
            param_types.push(param_type);
        }
        
        let return_type = self.type_checker.borrow().ast_type_to_type(&ext_func.return_type)?;
        let func_type = Type::function(param_types, return_type);
        
        // Check if external function already exists
        if let Some(existing_symbol) = self.symbol_table.lookup_symbol(&ext_func.name.name) {
            // For external functions, allow redeclaration if the signatures match
            if existing_symbol.kind == SymbolKind::Function {
                // Check if types match
                if !self.type_checker.borrow().types_compatible(&existing_symbol.symbol_type, &func_type) {
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
            symbol_type: func_type,
            kind: SymbolKind::Function, // External functions are treated as regular functions
            is_mutable: false,
            is_initialized: true,
            declaration_location: ext_func.source_location.clone(),
            is_moved: false,
            borrow_state: BorrowState::None,
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
    
    /// Get the symbol table (consumes the analyzer)
    pub fn get_symbol_table(self) -> SymbolTable {
        self.symbol_table
    }
    
    /// Get collected errors
    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }
    
    /// Check if analysis found any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Analyze a pattern and set up bindings
    fn analyze_pattern(&mut self, pattern: &Pattern, expected_type: &Type) -> Result<(), SemanticError> {
        match pattern {
            Pattern::EnumVariant { enum_name: _, variant_name, binding, nested_pattern, source_location } => {
                // Check that the pattern matches the expected enum type
                if let Type::Named { name: enum_type_name, .. } = expected_type {
                    // Find the enum definition
                    let enum_def = self.type_checker.borrow()
                        .lookup_type_definition(enum_type_name)
                        .cloned()
                        .ok_or_else(|| SemanticError::UndefinedSymbol {
                            symbol: enum_type_name.clone(),
                            location: source_location.clone(),
                        })?;
                    
                    if let crate::types::TypeDefinition::Enum { variants, .. } = enum_def {
                        // Find the matching variant
                        let variant = variants.iter()
                            .find(|v| v.name == variant_name.name)
                            .ok_or_else(|| SemanticError::UndefinedSymbol {
                                symbol: format!("variant '{}'", variant_name.name),
                                location: source_location.clone(),
                            })?;
                        
                        // Handle nested pattern
                        if let Some(ref nested_pat) = nested_pattern {
                            if let Some(ref associated_type) = variant.associated_type {
                                // Recursively analyze the nested pattern with the associated type
                                self.analyze_pattern(nested_pat, associated_type)?;
                            } else {
                                return Err(SemanticError::InvalidOperation {
                                    operation: "nested pattern matching".to_string(),
                                    reason: format!("variant '{}' has no associated data", variant_name.name),
                                    location: source_location.clone(),
                                });
                            }
                        }
                        
                        // If there's a binding (without nested pattern), add it to the symbol table
                        if let Some(binding_id) = binding {
                            if nested_pattern.is_none() {
                                if let Some(ref associated_type) = variant.associated_type {
                                    self.symbol_table.add_symbol(Symbol {
                                        name: binding_id.name.clone(),
                                        symbol_type: associated_type.clone(),
                                        kind: SymbolKind::Variable,
                                        is_mutable: false,
                                        is_initialized: true,
                                        declaration_location: binding_id.source_location.clone(),
                                        is_moved: false,
                                        borrow_state: BorrowState::None,
                                    })?;
                                }
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
                    })?;
                }
            }
            
            Pattern::Literal { .. } => {
                // Literal patterns don't create bindings
            }
        }
        
        Ok(())
    }
    
    /// Check if a set of match patterns is exhaustive for the given enum type
    fn check_match_exhaustiveness(&self, patterns: &[&Pattern], enum_type: &Type, location: &SourceLocation) -> Result<(), SemanticError> {
        // Extract the enum type name
        let enum_type_name = match enum_type {
            Type::Named { name, .. } => name,
            _ => return Ok(()), // Not an enum, skip exhaustiveness check
        };
        
        // Get the enum definition
        let enum_def = self.type_checker.borrow()
            .lookup_type_definition(enum_type_name)
            .cloned()
            .ok_or_else(|| SemanticError::UndefinedSymbol {
                symbol: enum_type_name.clone(),
                location: location.clone(),
            })?;
        
        if let crate::types::TypeDefinition::Enum { variants, .. } = enum_def {
            // Check if there's a wildcard pattern
            let has_wildcard = patterns.iter().any(|p| matches!(p, Pattern::Wildcard { .. }));
            
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
                    reason: format!("non-exhaustive patterns: missing variants {}", 
                        missing_variants.join(", ")),
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
mod tests {
    use super::*;
    use crate::error::SourceLocation;
    
    fn create_test_module() -> Module {
        Module {
            name: Identifier::new("test_module".to_string(), SourceLocation::unknown()),
            intent: Some("Test module".to_string()),
            imports: Vec::new(),
            exports: Vec::new(),
            type_definitions: Vec::new(),
            constant_declarations: vec![
                ConstantDeclaration {
                    name: Identifier::new("PI".to_string(), SourceLocation::unknown()),
                    type_spec: Box::new(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Float,
                        source_location: SourceLocation::unknown(),
                    }),
                    value: Box::new(Expression::FloatLiteral {
                        value: 3.14159,
                        source_location: SourceLocation::unknown(),
                    }),
                    intent: Some("Mathematical constant PI".to_string()),
                    source_location: SourceLocation::unknown(),
                }
            ],
            function_definitions: Vec::new(),
            external_functions: Vec::new(),
            source_location: SourceLocation::unknown(),
        }
    }
    
    #[test]
    fn test_semantic_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert!(!analyzer.has_errors());
        assert_eq!(analyzer.get_statistics().modules_analyzed, 0);
    }
    
    #[test]
    fn test_constant_declaration_analysis() {
        let mut analyzer = SemanticAnalyzer::new();
        let module = create_test_module();
        
        let result = analyzer.analyze_module(&module);
        assert!(result.is_ok());
        assert_eq!(analyzer.get_statistics().modules_analyzed, 1);
        assert_eq!(analyzer.get_statistics().variables_declared, 1);
    }
    
    #[test]
    fn test_type_mismatch_detection() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let mut module = create_test_module();
        // Change the constant to have mismatched type
        module.constant_declarations[0].value = Box::new(Expression::StringLiteral {
            value: "not a float".to_string(),
            source_location: SourceLocation::unknown(),
        });
        
        let result = analyzer.analyze_module(&module);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_expression_type_analysis() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Test integer literal
        let int_expr = Expression::IntegerLiteral {
            value: 42,
            source_location: SourceLocation::unknown(),
        };
        let int_type = analyzer.analyze_expression(&int_expr).unwrap();
        assert_eq!(int_type, Type::primitive(PrimitiveType::Integer));
        
        // Test arithmetic expression
        let add_expr = Expression::Add {
            left: Box::new(Expression::IntegerLiteral {
                value: 10,
                source_location: SourceLocation::unknown(),
            }),
            right: Box::new(Expression::IntegerLiteral {
                value: 20,
                source_location: SourceLocation::unknown(),
            }),
            source_location: SourceLocation::unknown(),
        };
        let add_type = analyzer.analyze_expression(&add_expr).unwrap();
        assert_eq!(add_type, Type::primitive(PrimitiveType::Integer));
    }
    
    #[test]
    fn test_variable_initialization_checking() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Add an uninitialized variable
        let var_symbol = Symbol::new(
            "x".to_string(),
            Type::primitive(PrimitiveType::Integer),
            SymbolKind::Variable,
            true,
            false,
            SourceLocation::unknown(),
        );
        
        analyzer.symbol_table.add_symbol(var_symbol).unwrap();
        
        // Try to use the uninitialized variable
        let var_expr = Expression::Variable {
            name: Identifier::new("x".to_string(), SourceLocation::unknown()),
            source_location: SourceLocation::unknown(),
        };
        
        let result = analyzer.analyze_expression(&var_expr);
        assert!(result.is_err());
        if let Err(SemanticError::UseBeforeInitialization { .. }) = result {
            // Expected error
        } else {
            panic!("Expected UseBeforeInitialization error");
        }
    }

    #[test]
    fn test_contract_validation_integration() {
        use crate::contracts::{ContractValidator, ContractContext};
        use crate::ast::{
            FunctionMetadata, ContractAssertion, FailureAction, PerformanceExpectation, 
            ComplexityExpectation, PerformanceMetric, ComplexityType, ComplexityNotation,
            Expression, PrimitiveType
        };
        use crate::types::{Type, TypeChecker};
        use crate::error::SourceLocation;
        use std::collections::HashMap;

        let mut validator = ContractValidator::new();
        let mut parameter_types = HashMap::new();
        parameter_types.insert("x".to_string(), Type::primitive(PrimitiveType::Integer));
        parameter_types.insert("y".to_string(), Type::primitive(PrimitiveType::Integer));
        
        let context = ContractContext {
            parameter_types,
            return_type: Type::primitive(PrimitiveType::Integer),
            type_checker: Rc::new(RefCell::new(TypeChecker::new())),
        };

        // Test valid metadata
        let valid_metadata = FunctionMetadata {
            preconditions: vec![ContractAssertion {
                condition: Box::new(Expression::BooleanLiteral {
                    value: true,
                    source_location: SourceLocation::unknown(),
                }),
                failure_action: FailureAction::AssertFail,
                message: Some("Test precondition".to_string()),
                source_location: SourceLocation::unknown(),
            }],
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: Some("division".to_string()),
            performance_expectation: Some(PerformanceExpectation {
                metric: PerformanceMetric::LatencyMs,
                target_value: 1.0,
                context: Some("Test latency".to_string()),
            }),
            complexity_expectation: Some(ComplexityExpectation {
                complexity_type: ComplexityType::Time,
                notation: ComplexityNotation::BigO,
                value: "O(1)".to_string(),
            }),
            throws_exceptions: Vec::new(),
            thread_safe: Some(true),
            may_block: Some(false),
        };

        let result = validator.validate_function_metadata(
            &valid_metadata,
            &context,
            "test_function",
            &SourceLocation::unknown(),
        );

        assert!(result.is_ok());
        let validation_result = result.unwrap();
        assert!(validation_result.is_valid);
        assert_eq!(validator.get_stats().functions_processed, 1);
        assert_eq!(validator.get_stats().preconditions_validated, 1);
        assert_eq!(validator.get_stats().performance_expectations_checked, 1);
        assert_eq!(validator.get_stats().complexity_expectations_checked, 1);
    }

    #[test]
    fn test_contract_validation_failures() {
        use crate::contracts::{ContractValidator, ContractContext};
        use crate::ast::{
            FunctionMetadata, PerformanceExpectation, ComplexityExpectation,
            PerformanceMetric, ComplexityType, ComplexityNotation
        };
        use crate::types::{Type, TypeChecker};
        use crate::error::SourceLocation;
        use std::collections::HashMap;

        let mut validator = ContractValidator::new();
        let context = ContractContext {
            parameter_types: HashMap::new(),
            return_type: Type::primitive(PrimitiveType::Void),
            type_checker: Rc::new(RefCell::new(TypeChecker::new())),
        };

        // Test invalid performance expectation
        let invalid_metadata = FunctionMetadata {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: None,
            performance_expectation: Some(PerformanceExpectation {
                metric: PerformanceMetric::LatencyMs,
                target_value: -10.0, // Invalid negative value
                context: None,
            }),
            complexity_expectation: Some(ComplexityExpectation {
                complexity_type: ComplexityType::Time,
                notation: ComplexityNotation::BigO,
                value: "O(invalid)".to_string(), // Invalid complexity notation
            }),
            throws_exceptions: Vec::new(),
            thread_safe: None,
            may_block: None,
        };

        let result = validator.validate_function_metadata(
            &invalid_metadata,
            &context,
            "bad_function",
            &SourceLocation::unknown(),
        );

        assert!(result.is_ok());
        let validation_result = result.unwrap();
        assert!(!validation_result.is_valid);
        assert!(!validation_result.errors.is_empty());
        assert_eq!(validator.get_stats().contract_errors, 2); // Performance + complexity errors
    }
}
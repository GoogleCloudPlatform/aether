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

//! Symbol table and scope management for AetherScript
//!
//! Handles variable and type symbol resolution with hierarchical scopes

use crate::error::{SemanticError, SourceLocation};
use crate::types::{Type, TypeDefinition};
use std::collections::HashMap;

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: Type,
    pub kind: SymbolKind,
    pub is_mutable: bool,
    pub is_initialized: bool,
    pub declaration_location: SourceLocation,
    /// Tracks if the value has been moved (ownership transferred)
    pub is_moved: bool,
    /// Tracks current borrow state
    pub borrow_state: BorrowState,
}

/// Borrow state of a symbol
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowState {
    /// Not borrowed
    None,
    /// Immutably borrowed (can have multiple)
    Borrowed(usize), // count of borrows
    /// Mutably borrowed (can only have one)
    BorrowedMut,
}

impl Symbol {
    /// Create a new symbol with default borrow state
    pub fn new(
        name: String,
        symbol_type: Type,
        kind: SymbolKind,
        is_mutable: bool,
        is_initialized: bool,
        declaration_location: SourceLocation,
    ) -> Self {
        Self {
            name,
            symbol_type,
            kind,
            is_mutable,
            is_initialized,
            declaration_location,
            is_moved: false,
            borrow_state: BorrowState::None,
        }
    }
}

/// Different kinds of symbols
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Constant,
    Function,
    Type,
    Module,
    Parameter,
}

/// Scope types for proper nesting
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeKind {
    Global,
    Module,
    Function,
    Block,
    Loop,
}

/// A scope containing symbols
#[derive(Debug, Clone)]
pub struct Scope {
    pub kind: ScopeKind,
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>, // Index into scope stack
    pub children: Vec<usize>,  // Indices of child scopes
}

impl Scope {
    /// Create a new scope
    pub fn new(kind: ScopeKind, parent: Option<usize>) -> Self {
        Self {
            kind,
            symbols: HashMap::new(),
            parent,
            children: Vec::new(),
        }
    }

    /// Add a symbol to this scope
    pub fn add_symbol(&mut self, symbol: Symbol) -> Result<(), SemanticError> {
        if self.symbols.contains_key(&symbol.name) {
            let existing = &self.symbols[&symbol.name];
            return Err(SemanticError::DuplicateDefinition {
                symbol: symbol.name,
                location: symbol.declaration_location,
                previous_location: existing.declaration_location.clone(),
            });
        }

        self.symbols.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Look up a symbol in this scope only
    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Get all symbols in this scope
    pub fn all_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }
}

/// Symbol table with hierarchical scopes
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Stack of scopes (index 0 is global scope)
    scopes: Vec<Scope>,

    /// Current scope index
    current_scope: usize,

    /// Type definitions
    type_definitions: HashMap<String, TypeDefinition>,

    /// Module imports mapping module names to their exported symbols
    imports: HashMap<String, HashMap<String, Symbol>>,

    /// Current module name
    current_module: Option<String>,
}

impl SymbolTable {
    /// Create a new symbol table with global scope
    pub fn new() -> Self {
        let global_scope = Scope::new(ScopeKind::Global, None);
        Self {
            scopes: vec![global_scope],
            current_scope: 0,
            type_definitions: HashMap::new(),
            imports: HashMap::new(),
            current_module: None,
        }
    }

    /// Set the current module
    pub fn set_current_module(&mut self, module_name: Option<String>) {
        self.current_module = module_name;
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self, kind: ScopeKind) -> usize {
        let new_scope_index = self.scopes.len();
        let new_scope = Scope::new(kind, Some(self.current_scope));

        // Add this scope as a child of the current scope
        self.scopes[self.current_scope]
            .children
            .push(new_scope_index);

        self.scopes.push(new_scope);
        self.current_scope = new_scope_index;

        new_scope_index
    }

    /// Exit the current scope, returning to parent
    pub fn exit_scope(&mut self) -> Result<(), SemanticError> {
        if self.current_scope == 0 {
            return Err(SemanticError::Internal {
                message: "Cannot exit global scope".to_string(),
            });
        }

        let parent = self.scopes[self.current_scope].parent.unwrap();
        self.current_scope = parent;
        Ok(())
    }

    /// Get the current scope
    pub fn current_scope(&self) -> &Scope {
        &self.scopes[self.current_scope]
    }

    /// Get a mutable reference to the current scope
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        &mut self.scopes[self.current_scope]
    }

    /// Add a symbol to the current scope
    pub fn add_symbol(&mut self, symbol: Symbol) -> Result<(), SemanticError> {
        self.current_scope_mut().add_symbol(symbol)
    }

    /// Look up a symbol, searching from current scope up to global
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        let mut current = self.current_scope;

        loop {
            if let Some(symbol) = self.scopes[current].lookup_local(name) {
                return Some(symbol);
            }

            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        // Check imports
        for imported_symbols in self.imports.values() {
            if let Some(symbol) = imported_symbols.get(name) {
                return Some(symbol);
            }
        }

        None
    }

    /// Look up a symbol in a specific scope only
    pub fn lookup_in_scope(&self, name: &str, scope_index: usize) -> Option<&Symbol> {
        if scope_index < self.scopes.len() {
            self.scopes[scope_index].lookup_local(name)
        } else {
            None
        }
    }

    /// Get all symbols across all scopes
    pub fn get_all_symbols(&self) -> Vec<&Symbol> {
        self.scopes.iter().flat_map(|scope| scope.symbols.values()).collect()
    }

    /// Add a type definition
    pub fn add_type_definition(
        &mut self,
        name: String,
        definition: TypeDefinition,
    ) -> Result<(), SemanticError> {
        if self.type_definitions.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: match &definition {
                    TypeDefinition::Struct {
                        source_location, ..
                    }
                    | TypeDefinition::Enum {
                        source_location, ..
                    }
                    | TypeDefinition::Alias {
                        source_location, ..
                    } => source_location.clone(),
                },
                previous_location: SourceLocation::unknown(), // TODO: Track previous location
            });
        }

        self.type_definitions.insert(name, definition);
        Ok(())
    }

    /// Look up a type definition
    pub fn lookup_type_definition(&self, name: &str) -> Option<&TypeDefinition> {
        self.type_definitions.get(name)
    }

    /// Get all type definitions
    pub fn get_type_definitions(&self) -> &HashMap<String, TypeDefinition> {
        &self.type_definitions
    }

    /// Add module imports
    pub fn add_import(&mut self, module_name: String, exported_symbols: HashMap<String, Symbol>) {
        self.imports.insert(module_name, exported_symbols);
    }

    /// Check if a variable has been initialized
    pub fn is_variable_initialized(&self, name: &str) -> bool {
        if let Some(symbol) = self.lookup_symbol(name) {
            symbol.is_initialized
        } else {
            false
        }
    }

    /// Mark a variable as initialized
    pub fn mark_variable_initialized(&mut self, name: &str) -> Result<(), SemanticError> {
        // Search through scopes to find and mark the variable
        let mut current = self.current_scope;

        loop {
            if let Some(symbol) = self.scopes[current].symbols.get_mut(name) {
                symbol.is_initialized = true;
                return Ok(());
            }

            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        Err(SemanticError::UndefinedSymbol {
            symbol: name.to_string(),
            location: SourceLocation::unknown(),
        })
    }

    /// Check if a variable is mutable
    pub fn is_variable_mutable(&self, name: &str) -> Result<bool, SemanticError> {
        if let Some(symbol) = self.lookup_symbol(name) {
            Ok(symbol.is_mutable)
        } else {
            Err(SemanticError::UndefinedSymbol {
                symbol: name.to_string(),
                location: SourceLocation::unknown(),
            })
        }
    }

    /// Get all symbols in the current scope
    pub fn current_scope_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.current_scope().all_symbols()
    }

    /// Get all symbols visible from the current scope
    pub fn visible_symbols(&self) -> Vec<&Symbol> {
        let mut symbols = Vec::new();
        let mut current = self.current_scope;

        // Collect symbols from current scope up to global
        loop {
            for symbol in self.scopes[current].all_symbols() {
                symbols.push(symbol);
            }

            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        // Add imported symbols
        for imported_symbols in self.imports.values() {
            for symbol in imported_symbols.values() {
                symbols.push(symbol);
            }
        }

        symbols
    }

    /// Check for unused variables in the current scope
    pub fn find_unused_variables(&self) -> Vec<&Symbol> {
        // This is a simple implementation - a more sophisticated one would track usage
        self.current_scope()
            .all_symbols()
            .filter(|symbol| symbol.kind == SymbolKind::Variable && !symbol.name.starts_with('_'))
            .collect()
    }

    /// Get scope depth (0 = global, 1 = module, 2 = function, etc.)
    pub fn scope_depth(&self) -> usize {
        let mut depth = 0;
        let mut current = self.current_scope;

        while let Some(parent) = self.scopes[current].parent {
            depth += 1;
            current = parent;
        }

        depth
    }

    /// Check if we're in a specific scope kind
    pub fn in_scope_kind(&self, kind: ScopeKind) -> bool {
        let mut current = self.current_scope;

        loop {
            if self.scopes[current].kind == kind {
                return true;
            }

            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        false
    }

    /// Find the nearest scope of a specific kind
    pub fn find_nearest_scope(&self, kind: ScopeKind) -> Option<usize> {
        let mut current = self.current_scope;

        loop {
            if self.scopes[current].kind == kind {
                return Some(current);
            }

            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        None
    }

    /// Mark a variable as moved (ownership transferred)
    pub fn mark_variable_moved(&mut self, name: &str) -> Result<(), SemanticError> {
        let mut current = self.current_scope;

        loop {
            if let Some(symbol) = self.scopes[current].symbols.get_mut(name) {
                symbol.is_moved = true;
                return Ok(());
            }

            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        Err(SemanticError::UndefinedSymbol {
            symbol: name.to_string(),
            location: SourceLocation::unknown(),
        })
    }

    /// Borrow a variable immutably
    pub fn borrow_variable(&mut self, name: &str) -> Result<(), SemanticError> {
        let mut current = self.current_scope;

        loop {
            if let Some(symbol) = self.scopes[current].symbols.get_mut(name) {
                match &mut symbol.borrow_state {
                    BorrowState::None => {
                        symbol.borrow_state = BorrowState::Borrowed(1);
                        return Ok(());
                    }
                    BorrowState::Borrowed(count) => {
                        *count += 1;
                        return Ok(());
                    }
                    BorrowState::BorrowedMut => {
                        return Err(SemanticError::InvalidOperation {
                            operation: "immutable borrow".to_string(),
                            reason: "variable is already mutably borrowed".to_string(),
                            location: SourceLocation::unknown(),
                        });
                    }
                }
            }

            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        Err(SemanticError::UndefinedSymbol {
            symbol: name.to_string(),
            location: SourceLocation::unknown(),
        })
    }

    /// Borrow a variable mutably
    pub fn borrow_variable_mut(&mut self, name: &str) -> Result<(), SemanticError> {
        let mut current = self.current_scope;

        loop {
            if let Some(symbol) = self.scopes[current].symbols.get_mut(name) {
                if !symbol.is_mutable {
                    return Err(SemanticError::AssignToImmutable {
                        variable: name.to_string(),
                        location: SourceLocation::unknown(),
                    });
                }

                match &symbol.borrow_state {
                    BorrowState::None => {
                        symbol.borrow_state = BorrowState::BorrowedMut;
                        return Ok(());
                    }
                    BorrowState::Borrowed(_) => {
                        return Err(SemanticError::InvalidOperation {
                            operation: "mutable borrow".to_string(),
                            reason: "variable is already immutably borrowed".to_string(),
                            location: SourceLocation::unknown(),
                        });
                    }
                    BorrowState::BorrowedMut => {
                        return Err(SemanticError::InvalidOperation {
                            operation: "mutable borrow".to_string(),
                            reason: "variable is already mutably borrowed".to_string(),
                            location: SourceLocation::unknown(),
                        });
                    }
                }
            }

            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        Err(SemanticError::UndefinedSymbol {
            symbol: name.to_string(),
            location: SourceLocation::unknown(),
        })
    }

    /// Release a borrow
    pub fn release_borrow(&mut self, name: &str) -> Result<(), SemanticError> {
        let mut current = self.current_scope;

        loop {
            if let Some(symbol) = self.scopes[current].symbols.get_mut(name) {
                match &mut symbol.borrow_state {
                    BorrowState::None => {
                        return Err(SemanticError::InvalidOperation {
                            operation: "release borrow".to_string(),
                            reason: "variable is not borrowed".to_string(),
                            location: SourceLocation::unknown(),
                        });
                    }
                    BorrowState::Borrowed(count) => {
                        if *count > 1 {
                            *count -= 1;
                        } else {
                            symbol.borrow_state = BorrowState::None;
                        }
                        return Ok(());
                    }
                    BorrowState::BorrowedMut => {
                        symbol.borrow_state = BorrowState::None;
                        return Ok(());
                    }
                }
            }

            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        Err(SemanticError::UndefinedSymbol {
            symbol: name.to_string(),
            location: SourceLocation::unknown(),
        })
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;
    use crate::types::Type;

    fn create_test_symbol(name: &str, symbol_type: Type) -> Symbol {
        Symbol::new(
            name.to_string(),
            symbol_type,
            SymbolKind::Variable,
            true,
            true,
            SourceLocation::unknown(),
        )
    }

    #[test]
    fn test_symbol_table_creation() {
        let table = SymbolTable::new();
        assert_eq!(table.current_scope, 0);
        assert_eq!(table.scopes.len(), 1);
        assert_eq!(table.scopes[0].kind, ScopeKind::Global);
    }

    #[test]
    fn test_scope_management() {
        let mut table = SymbolTable::new();

        // Enter function scope
        let func_scope = table.enter_scope(ScopeKind::Function);
        assert_eq!(func_scope, 1);
        assert_eq!(table.current_scope, 1);
        assert_eq!(table.scopes.len(), 2);

        // Enter block scope
        let block_scope = table.enter_scope(ScopeKind::Block);
        assert_eq!(block_scope, 2);
        assert_eq!(table.current_scope, 2);

        // Exit back to function scope
        assert!(table.exit_scope().is_ok());
        assert_eq!(table.current_scope, 1);

        // Exit back to global scope
        assert!(table.exit_scope().is_ok());
        assert_eq!(table.current_scope, 0);

        // Cannot exit global scope
        assert!(table.exit_scope().is_err());
    }

    #[test]
    fn test_symbol_addition_and_lookup() {
        let mut table = SymbolTable::new();

        let symbol = create_test_symbol("x", Type::primitive(PrimitiveType::Integer));
        assert!(table.add_symbol(symbol).is_ok());

        // Should find the symbol
        assert!(table.lookup_symbol("x").is_some());
        assert!(table.lookup_symbol("y").is_none());

        // Test duplicate symbol error
        let duplicate = create_test_symbol("x", Type::primitive(PrimitiveType::Float));
        assert!(table.add_symbol(duplicate).is_err());
    }

    #[test]
    fn test_hierarchical_lookup() {
        let mut table = SymbolTable::new();

        // Add symbol in global scope
        let global_symbol =
            create_test_symbol("global_var", Type::primitive(PrimitiveType::String));
        assert!(table.add_symbol(global_symbol).is_ok());

        // Enter function scope
        table.enter_scope(ScopeKind::Function);

        // Add symbol in function scope
        let local_symbol = create_test_symbol("local_var", Type::primitive(PrimitiveType::Integer));
        assert!(table.add_symbol(local_symbol).is_ok());

        // Should find both symbols
        assert!(table.lookup_symbol("global_var").is_some());
        assert!(table.lookup_symbol("local_var").is_some());

        // Enter block scope
        table.enter_scope(ScopeKind::Block);

        // Should still find both symbols
        assert!(table.lookup_symbol("global_var").is_some());
        assert!(table.lookup_symbol("local_var").is_some());

        // Add symbol that shadows global
        let shadow_symbol =
            create_test_symbol("global_var", Type::primitive(PrimitiveType::Boolean));
        assert!(table.add_symbol(shadow_symbol).is_ok());

        // Should find the shadowing symbol
        let found = table.lookup_symbol("global_var").unwrap();
        assert_eq!(found.symbol_type, Type::primitive(PrimitiveType::Boolean));
    }

    #[test]
    fn test_variable_initialization_tracking() {
        let mut table = SymbolTable::new();

        let mut symbol = create_test_symbol("x", Type::primitive(PrimitiveType::Integer));
        symbol.is_initialized = false;
        assert!(table.add_symbol(symbol).is_ok());

        // Variable should not be initialized
        assert!(!table.is_variable_initialized("x"));

        // Mark as initialized
        assert!(table.mark_variable_initialized("x").is_ok());
        assert!(table.is_variable_initialized("x"));

        // Test with non-existent variable
        assert!(table.mark_variable_initialized("y").is_err());
    }

    #[test]
    fn test_type_definitions() {
        let mut table = SymbolTable::new();

        let type_def = TypeDefinition::Alias {
            target_type: Type::primitive(PrimitiveType::Integer),
            source_location: SourceLocation::unknown(),
        };

        assert!(table
            .add_type_definition("MyInt".to_string(), type_def)
            .is_ok());
        assert!(table.lookup_type_definition("MyInt").is_some());
        assert!(table.lookup_type_definition("Unknown").is_none());

        // Test duplicate type definition
        let duplicate_def = TypeDefinition::Alias {
            target_type: Type::primitive(PrimitiveType::Float),
            source_location: SourceLocation::unknown(),
        };
        assert!(table
            .add_type_definition("MyInt".to_string(), duplicate_def)
            .is_err());
    }

    #[test]
    fn test_scope_kind_checking() {
        let mut table = SymbolTable::new();

        assert!(table.in_scope_kind(ScopeKind::Global));
        assert!(!table.in_scope_kind(ScopeKind::Function));

        table.enter_scope(ScopeKind::Function);
        assert!(table.in_scope_kind(ScopeKind::Global));
        assert!(table.in_scope_kind(ScopeKind::Function));

        table.enter_scope(ScopeKind::Block);
        assert!(table.in_scope_kind(ScopeKind::Global));
        assert!(table.in_scope_kind(ScopeKind::Function));
        assert!(table.in_scope_kind(ScopeKind::Block));

        // Test finding nearest scope
        assert_eq!(table.find_nearest_scope(ScopeKind::Block), Some(2));
        assert_eq!(table.find_nearest_scope(ScopeKind::Function), Some(1));
        assert_eq!(table.find_nearest_scope(ScopeKind::Global), Some(0));
        assert_eq!(table.find_nearest_scope(ScopeKind::Loop), None);
    }
}

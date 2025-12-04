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

use crate::ast::Expression;
use crate::error::SemanticError;
use crate::semantic::SemanticAnalyzer;
use crate::types::{OwnershipKind, Type};

impl SemanticAnalyzer {
    /// Check ownership transfer for function arguments
    pub fn check_argument_ownership(
        &mut self,
        arg_expr: &Expression,
        param_type: &Type,
    ) -> Result<(), SemanticError> {
        eprintln!(
            "Checking ownership for arg: {:?}, param_type: {:?}",
            arg_expr, param_type
        );
        if let Expression::Variable {
            name,
            source_location,
        } = arg_expr
        {
            // Check if parameter type expects ownership transfer or borrowing
            match param_type {
                Type::Owned {
                    ownership: OwnershipKind::Owned,
                    ..
                } => {
                    eprintln!("Marking variable {} as moved", name.name);
                    // Transfer ownership (move)
                    if let Err(_e) = self.symbol_table.mark_variable_moved(&name.name) {
                        return Err(SemanticError::UseAfterMove {
                            variable: name.name.clone(),
                            location: source_location.clone(),
                        });
                    }
                }
                Type::Owned {
                    ownership: OwnershipKind::Borrowed,
                    ..
                } => {
                    // Immutable borrow
                    self.symbol_table.borrow_variable(&name.name)?;
                }
                Type::Owned {
                    ownership: OwnershipKind::MutableBorrow,
                    ..
                } => {
                    // Mutable borrow
                    self.symbol_table.borrow_variable_mut(&name.name)?;
                }
                _ => {
                    // For types not explicitly wrapped in Owned, default to Move for non-primitives
                    // and Copy for primitives.
                    // Functions are also Copy (pointers).
                    // Types with @derive(Copy) are also Copy.
                    let is_copy_type = param_type.is_primitive()
                        || matches!(param_type, Type::Function { .. })
                        || self.is_copy_type(param_type);

                    if !is_copy_type {
                        if let Err(_e) = self.symbol_table.mark_variable_moved(&name.name) {
                            // Try qualified lookup
                            let qualified_name = if let Some(module) = &self.current_module {
                                format!("{}.{}", module, name.name)
                            } else {
                                name.name.clone()
                            };

                            if let Err(_) = self.symbol_table.mark_variable_moved(&qualified_name) {
                                return Err(SemanticError::UseAfterMove {
                                    variable: name.name.clone(),
                                    location: source_location.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if a type has the Copy trait (via @derive(Copy))
    fn is_copy_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Named { name, module } => {
                // Look up the type definition
                let full_name = if let Some(m) = module {
                    format!("{}.{}", m, name)
                } else if let Some(m) = &self.current_module {
                    format!("{}.{}", m, name)
                } else {
                    name.clone()
                };

                let type_defs = self.symbol_table.get_type_definitions();

                // Try to get the type definition from symbol table
                if let Some(type_def) = type_defs.get(&full_name) {
                    if let crate::types::TypeDefinition::Struct { is_copy, .. } = type_def {
                        return *is_copy;
                    }
                }
                // Also try without module prefix
                if let Some(type_def) = type_defs.get(name) {
                    if let crate::types::TypeDefinition::Struct { is_copy, .. } = type_def {
                        return *is_copy;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

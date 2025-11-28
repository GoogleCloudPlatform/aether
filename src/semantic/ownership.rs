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
        eprintln!("Checking ownership for arg: {:?}, param_type: {:?}", arg_expr, param_type);
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
                    if let Err(e) = self.symbol_table.mark_variable_moved(&name.name) {
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
                    if !param_type.is_primitive() {
                        if let Err(e) = self.symbol_table.mark_variable_moved(&name.name) {
                            return Err(SemanticError::UseAfterMove {
                                variable: name.name.clone(),
                                location: source_location.clone(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

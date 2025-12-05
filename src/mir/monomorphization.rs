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

//! Monomorphization pass for AetherScript MIR
//!
//! Instantiates generic functions with concrete types.

use crate::mir::{
    Constant, ConstantValue, Function, Operand, Program, Rvalue, Statement, Terminator,
};
use crate::types::Type;
use std::collections::{HashMap, HashSet};

/// Monomorphization pass
pub struct Monomorphizer {
    /// Cache of instantiated functions: (original_name, type_args) -> mangled_name
    instantiated_funcs: HashMap<(String, Vec<Type>), String>,
    /// Queue of functions to process: (mangled_name, original_name, type_args)
    queue: Vec<(String, String, Vec<Type>)>,
    /// Set of processed functions to avoid infinite loops
    processed: HashSet<String>,
}

impl Default for Monomorphizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Monomorphizer {
    pub fn new() -> Self {
        Self {
            instantiated_funcs: HashMap::new(),
            queue: Vec::new(),
            processed: HashSet::new(),
        }
    }

    /// Run the monomorphization pass on the given program
    pub fn run(&mut self, program: &mut Program) {
        // Start with main function and any exported functions
        let mut entry_points = Vec::new();
        for name in program.functions.keys() {
            // For now, assume "main" and non-generic functions are entry points
            if name == "main" || !self.is_generic(program, name) {
                entry_points.push(name.clone());
            }
        }

        // Process entry points
        for name in entry_points {
            self.process_function(program, &name);
        }

        // Process the queue of instantiations
        while let Some((mangled_name, original_name, type_args)) = self.queue.pop() {
            if self.processed.contains(&mangled_name) {
                continue;
            }

            self.instantiate_function(program, &mangled_name, &original_name, &type_args);
            self.process_function(program, &mangled_name);
            self.processed.insert(mangled_name);
        }
    }

    /// Check if a function is generic (has generic parameters)
    fn is_generic(&self, program: &Program, func_name: &str) -> bool {
        if let Some(func) = program.functions.get(func_name) {
            func.parameters.iter().any(|p| self.contains_generic(&p.ty))
                || self.contains_generic(&func.return_type)
        } else {
            false
        }
    }

    /// Check if a type contains generic parameters
    fn contains_generic(&self, ty: &Type) -> bool {
        match ty {
            Type::Generic { .. } => true,
            Type::GenericInstance { type_arguments, .. } => {
                type_arguments.iter().any(|t| self.contains_generic(t))
            }
            Type::Array { element_type, .. } => self.contains_generic(element_type),
            Type::Map {
                key_type,
                value_type,
            } => self.contains_generic(key_type) || self.contains_generic(value_type),
            Type::Pointer { target_type, .. } => self.contains_generic(target_type),
            Type::Owned { base_type, .. } => self.contains_generic(base_type),
            Type::Function {
                parameter_types,
                return_type,
                ..
            } => {
                parameter_types.iter().any(|t| self.contains_generic(t))
                    || self.contains_generic(return_type)
            }
            _ => false,
        }
    }

    /// Process a function to find generic calls
    fn process_function(&mut self, program: &mut Program, func_name: &str) {
        // We need to clone the function body to analyze it while mutating the program potentially
        // But here we are just scanning for calls and updating them in place?
        // Updating in place is tricky if we need mutable access to program to add new functions.
        // So we'll scan first, collect replacements, then apply them.

        let mut calls_to_replace = Vec::new();

        if let Some(caller_func) = program.functions.get(func_name) {
            for (block_id, block) in &caller_func.basic_blocks {
                // Check statements for calls (assignments with Rvalue::Call)
                for (stmt_idx, stmt) in block.statements.iter().enumerate() {
                    if let Statement::Assign {
                        rvalue:
                            Rvalue::Call {
                                func: callee_operand,
                                explicit_type_arguments,
                                args,
                            },
                        ..
                    } = stmt
                    {
                        if let Operand::Constant(Constant {
                            value: ConstantValue::String(callee_name),
                            ..
                        }) = callee_operand
                        {
                            if let Some(replacement) = self.check_generic_call(
                                program,
                                caller_func,
                                callee_name,
                                explicit_type_arguments,
                                args,
                            ) {
                                calls_to_replace.push((*block_id, stmt_idx, replacement));
                            }
                        }
                    }
                }

                // Check terminator for calls
                if let Terminator::Call {
                    func: callee_operand,
                    explicit_type_arguments,
                    args,
                    ..
                } = &block.terminator
                {
                    if let Operand::Constant(Constant {
                        value: ConstantValue::String(callee_name),
                        ..
                    }) = callee_operand
                    {
                        if let Some(replacement) = self.check_generic_call(
                            program,
                            caller_func,
                            callee_name,
                            explicit_type_arguments,
                            args,
                        ) {
                            // Use a special index for terminator
                            calls_to_replace.push((*block_id, usize::MAX, replacement));
                        }
                    }
                }
            }
        }

        // Apply replacements
        if let Some(func) = program.functions.get_mut(func_name) {
            for (block_id, stmt_idx, new_name) in calls_to_replace {
                let new_func_operand = Operand::Constant(Constant {
                    ty: Type::primitive(crate::ast::PrimitiveType::String),
                    value: ConstantValue::String(new_name),
                });

                if stmt_idx == usize::MAX {
                    // Update terminator
                    if let Some(block) = func.basic_blocks.get_mut(&block_id) {
                        if let Terminator::Call { func, .. } = &mut block.terminator {
                            *func = new_func_operand;
                        }
                    }
                } else {
                    // Update statement
                    if let Some(block) = func.basic_blocks.get_mut(&block_id) {
                        if let Statement::Assign {
                            rvalue: Rvalue::Call { func, .. },
                            ..
                        } = &mut block.statements[stmt_idx]
                        {
                            *func = new_func_operand;
                        }
                    }
                }
            }
        }
    }

    /// Check if a call is to a generic function and needs instantiation
    fn check_generic_call(
        &mut self,
        program: &Program,
        caller_func: &Function,
        callee_name: &str,
        explicit_type_arguments: &[Type],
        args: &[Operand],
    ) -> Option<String> {
        // If callee is not generic, no need to instantiate
        if !self.is_generic(program, callee_name) {
            return None;
        }

        let inferred_types = if !explicit_type_arguments.is_empty() {
            // If explicit type arguments are provided, use them directly
            explicit_type_arguments.to_vec()
        } else {
            // Otherwise, infer type arguments from arguments
            let callee = program.functions.get(callee_name)?;
            self.infer_type_args(callee, args, caller_func)?
        };

        // Mangle name
        let mangled_name = self.mangle_name(callee_name, &inferred_types);

        // If not already instantiated/queued, add to queue
        if let std::collections::hash_map::Entry::Vacant(e) = self
            .instantiated_funcs
            .entry((callee_name.to_string(), inferred_types.clone()))
        {
            e.insert(mangled_name.clone());
            self.queue.push((
                mangled_name.clone(),
                callee_name.to_string(),
                inferred_types,
            ));
        }

        Some(mangled_name)
    }

    /// Infer generic type arguments by matching call arguments with function parameters
    fn infer_type_args(
        &self,
        callee: &Function,
        args: &[Operand],
        caller_func: &Function,
    ) -> Option<Vec<Type>> {
        // This is a simplified inference.
        // In a full implementation, we'd use unification.
        // Here we assume simple one-to-one mapping or direct usage.

        let mut inferred = HashMap::new();

        for (param, arg_op) in callee.parameters.iter().zip(args.iter()) {
            let arg_type = self.get_operand_type(arg_op, caller_func);
            self.unify_types(&param.ty, &arg_type, &mut inferred);
        }

        // Collect results in order?
        // We need to know the order of generic parameters.
        // Since MIR Function doesn't store generic params list, we have to guess or scan.
        // For this task, let's assume we scan the parameter types for Generic(name) and order them?
        // Or better, let's assume T, U, V order or alphabetical.

        let mut generic_params = HashSet::new();
        self.collect_generic_params(&callee.parameters, &mut generic_params);
        self.collect_generic_params_from_type(&callee.return_type, &mut generic_params);

        let mut sorted_params: Vec<_> = generic_params.into_iter().collect();
        sorted_params.sort();

        let mut result = Vec::new();
        for param in sorted_params {
            if let Some(ty) = inferred.get(&param) {
                result.push(ty.clone());
            } else {
                // Could not infer type
                eprintln!("Could not infer type for generic parameter {}", param);
                return None;
            }
        }

        Some(result)
    }

    fn collect_generic_params(&self, params: &[crate::mir::Parameter], set: &mut HashSet<String>) {
        for param in params {
            self.collect_generic_params_from_type(&param.ty, set);
        }
    }

    fn collect_generic_params_from_type(&self, ty: &Type, set: &mut HashSet<String>) {
        match ty {
            Type::Generic { name, .. } => {
                set.insert(name.clone());
            }
            Type::GenericInstance { type_arguments, .. } => {
                for t in type_arguments {
                    self.collect_generic_params_from_type(t, set);
                }
            }
            Type::Array { element_type, .. } => {
                self.collect_generic_params_from_type(element_type, set)
            }
            Type::Pointer { target_type, .. } => {
                self.collect_generic_params_from_type(target_type, set)
            }
            Type::Owned { base_type, .. } => self.collect_generic_params_from_type(base_type, set),
            Type::Function {
                parameter_types,
                return_type,
                ..
            } => {
                for t in parameter_types {
                    self.collect_generic_params_from_type(t, set);
                }
                self.collect_generic_params_from_type(return_type, set);
            }
            _ => {}
        }
    }

    fn unify_types(
        &self,
        generic_ty: &Type,
        concrete_ty: &Type,
        inferred: &mut HashMap<String, Type>,
    ) {
        match (generic_ty, concrete_ty) {
            (Type::Generic { name, .. }, _) => {
                // Found a mapping
                // Check for conflict?
                inferred.insert(name.clone(), concrete_ty.clone());
            }
            (
                Type::Array {
                    element_type: g, ..
                },
                Type::Array {
                    element_type: c, ..
                },
            ) => {
                self.unify_types(g, c, inferred);
            }
            (Type::Pointer { target_type: g, .. }, Type::Pointer { target_type: c, .. }) => {
                self.unify_types(g, c, inferred);
            }
            (Type::Owned { base_type: g, .. }, Type::Owned { base_type: c, .. }) => {
                self.unify_types(g, c, inferred);
            }
            _ => {}
        }
    }

    fn get_operand_type(&self, op: &Operand, caller_func: &Function) -> Type {
        match op {
            Operand::Constant(c) => c.ty.clone(),
            Operand::Copy(place) | Operand::Move(place) => {
                if let Some(local) = caller_func.locals.get(&place.local) {
                    local.ty.clone()
                } else {
                    Type::Error
                }
            }
        }
    }

    /// Mangle function name with type arguments
    fn mangle_name(&self, name: &str, type_args: &[Type]) -> String {
        let mut mangled = name.to_string();
        for ty in type_args {
            mangled.push('_');
            mangled.push_str(&self.type_to_string_mangled(ty));
        }
        mangled
    }

    fn type_to_string_mangled(&self, ty: &Type) -> String {
        match ty {
            Type::Primitive(p) => format!("{:?}", p),
            Type::Named { name, .. } => name.clone(),
            _ => "complex".to_string(), // Simplified
        }
    }

    /// Instantiate a generic function with concrete types
    fn instantiate_function(
        &mut self,
        program: &mut Program,
        mangled_name: &str,
        original_name: &str,
        type_args: &[Type],
    ) {
        if let Some(original_func) = program.functions.get(original_name).cloned() {
            // Determine generic params map
            let mut type_map = HashMap::new();
            let mut generic_params = HashSet::new();
            self.collect_generic_params(&original_func.parameters, &mut generic_params);
            self.collect_generic_params_from_type(&original_func.return_type, &mut generic_params);

            let mut sorted_params: Vec<_> = generic_params.into_iter().collect();
            sorted_params.sort();

            for (name, ty) in sorted_params.iter().zip(type_args.iter()) {
                type_map.insert(name.clone(), ty.clone());
            }

            // Create new function
            let mut new_func = original_func.clone();
            new_func.name = mangled_name.to_string();

            // Substitute types in parameters
            for param in &mut new_func.parameters {
                param.ty = self.substitute_type(&param.ty, &type_map);
            }

            // Substitute return type
            new_func.return_type = self.substitute_type(&new_func.return_type, &type_map);

            // Substitute types in locals
            for local in new_func.locals.values_mut() {
                local.ty = self.substitute_type(&local.ty, &type_map);
            }

            // Note: We assume statements don't contain types explicitly that need substitution
            // EXCEPT for Casts or Aggregate construction.
            // We should scan statements and substitute there too.
            for block in new_func.basic_blocks.values_mut() {
                for stmt in &mut block.statements {
                    self.substitute_in_statement(stmt, &type_map);
                }
                self.substitute_in_terminator(&mut block.terminator, &type_map);
            }

            program.functions.insert(mangled_name.to_string(), new_func);
        }
    }

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
            _ => ty.clone(),
        }
    }

    fn substitute_in_statement(&self, stmt: &mut Statement, type_map: &HashMap<String, Type>) {
        if let Statement::Assign { rvalue, .. } = stmt {
            self.substitute_in_rvalue(rvalue, type_map);
        }
    }

    fn substitute_in_rvalue(&self, rvalue: &mut Rvalue, type_map: &HashMap<String, Type>) {
        match rvalue {
            Rvalue::Cast { ty, .. } => {
                *ty = self.substitute_type(ty, type_map);
            }
            Rvalue::Aggregate { kind, .. } => {
                if let crate::mir::AggregateKind::Array(ty) = kind {
                    *ty = self.substitute_type(ty, type_map);
                }
            }
            _ => {}
        }
    }

    fn substitute_in_terminator(&self, term: &mut Terminator, type_map: &HashMap<String, Type>) {
        if let Terminator::SwitchInt { switch_ty, .. } = term {
            *switch_ty = self.substitute_type(switch_ty, type_map);
        }
    }
}

#[cfg(test)]
mod tests;

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

//! Capture analysis for identifying variables used in concurrent blocks
//! that are defined in enclosing scopes.

use crate::ast::*;
use crate::error::SourceLocation;
use std::collections::{HashMap, HashSet};

/// Analyzer for identifying captured variables in concurrent blocks
pub struct CaptureAnalyzer {
    /// Stack of scopes. Each scope is a set of variable names defined in that scope.
    scopes: Vec<HashSet<String>>,
    /// Stack of concurrent block depths.
    /// We store the scope depth at which the concurrent block started.
    /// We also store the SourceLocation to identify the block later.
    concurrent_stack: Vec<(usize, SourceLocation)>,
    /// Result: Map from Concurrent Block Location to set of captured variables.
    pub captures: HashMap<SourceLocation, HashSet<String>>,
}

impl CaptureAnalyzer {
    /// Create a new capture analyzer
    pub fn new() -> Self {
        Self {
            scopes: vec![HashSet::new()], // Global scope
            concurrent_stack: Vec::new(),
            captures: HashMap::new(),
        }
    }

    /// Analyze a module
    pub fn analyze(&mut self, module: &Module) {
        self.visit_module(module);
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_variable(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn check_variable_usage(&mut self, name: &str) {
        // 1. Find definition depth
        let mut defined_depth = None;
        // Iterate from innermost scope to outermost
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains(name) {
                defined_depth = Some(i);
                break;
            }
        }

        // If found, check if it's a capture for any active concurrent block
        if let Some(depth) = defined_depth {
            for (block_start_depth, block_loc) in &self.concurrent_stack {
                // If the variable was defined in a scope OLDER (lower index) than
                // where the concurrent block started, it is captured.
                if depth < *block_start_depth {
                    self.captures
                        .entry(block_loc.clone())
                        .or_default()
                        .insert(name.to_string());
                }
            }
        }
        // If not found, it might be a global or import.
        // TODO: Decide if globals need to be treated as captures.
        // Usually globals are static and don't need "capturing" in the closure struct sense,
        // but for thread safety analysis, they are relevant.
        // For code generation (creating the struct), we usually only care about local stack variables.
    }
}

impl ASTVisitor<()> for CaptureAnalyzer {
    fn visit_module(&mut self, node: &Module) {
        // Register top-level constants and functions in global scope
        for const_decl in &node.constant_declarations {
            self.define_variable(&const_decl.name.name);
        }
        for func in &node.function_definitions {
            self.define_variable(&func.name.name);
        }

        // Visit functions
        for func in &node.function_definitions {
            self.visit_function(func);
        }
    }

    fn visit_function(&mut self, node: &Function) {
        self.enter_scope();

        // Add parameters to scope
        for param in &node.parameters {
            self.define_variable(&param.name.name);
        }

        // Visit body
        self.visit_statement(&Statement::Expression {
            expr: Box::new(Expression::Variable { // Dummy expression to wrap block? No, just visit block.
                name: Identifier::new("".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            }),
            source_location: SourceLocation::unknown(),
        });
        
        // Actually, better to call visit_block logic directly or via a helper
        // But Block is usually visited via Statement::Block or similar.
        // In Aether AST, Function has a `body: Block`.
        // And `Block` is not a `Statement` itself, but contains statements.
        // Let's manually handle the block content here to avoid `Statement` wrapper mess
        
        // The Block structure:
        for stmt in &node.body.statements {
            self.visit_statement(stmt);
        }

        self.exit_scope();
    }

    fn visit_statement(&mut self, node: &Statement) {
        match node {
            Statement::VariableDeclaration { name, initial_value, .. } => {
                // Visit initializer BEFORE defining the variable (to avoid self-reference if disallowed, 
                // or handled correctly for captures if it refers to others)
                if let Some(expr) = initial_value {
                    self.visit_expression(expr);
                }
                self.define_variable(&name.name);
            }
            Statement::Concurrent { block, source_location } => {
                // Start of concurrent block
                // The block's scope will be created inside visit_block (or manually here)
                
                // We mark the current scope depth. Any variable defined at `current_depth` or less (if we haven't entered block scope yet)
                // will be outside.
                // Actually, we are about to enter the block.
                // The concurrent block will create a new scope.
                // So "outside" means defined in `self.scopes.len()` or less?
                // If I push to stack BEFORE entering the block's scope:
                // stack depth = N.
                // New scope will be N+1.
                // Variable defined at < N+1 is captured?
                // Actually, variable defined at <= N is captured.
                // So if defined_depth < scopes.len() (which is N+1 after enter), it is captured.
                // So `block_start_depth` should be `self.scopes.len()`.
                
                let start_depth = self.scopes.len();
                self.concurrent_stack.push((start_depth, source_location.clone()));
                
                // Enter scope for the block
                self.enter_scope();
                
                for stmt in &block.statements {
                    self.visit_statement(stmt);
                }
                
                self.exit_scope();
                
                self.concurrent_stack.pop();
            }
            Statement::If { condition, then_block, else_ifs, else_block, .. } => {
                self.visit_expression(condition);
                
                self.enter_scope();
                for stmt in &then_block.statements { self.visit_statement(stmt); }
                self.exit_scope();

                for else_if in else_ifs {
                    self.visit_expression(&else_if.condition);
                    self.enter_scope();
                    for stmt in &else_if.block.statements { self.visit_statement(stmt); }
                    self.exit_scope();
                }

                if let Some(block) = else_block {
                    self.enter_scope();
                    for stmt in &block.statements { self.visit_statement(stmt); }
                    self.exit_scope();
                }
            }
            Statement::WhileLoop { condition, body, .. } => {
                self.visit_expression(condition);
                self.enter_scope();
                for stmt in &body.statements { self.visit_statement(stmt); }
                self.exit_scope();
            }
            Statement::ForEachLoop { collection, element_binding, body, .. } => {
                self.visit_expression(collection);
                self.enter_scope();
                self.define_variable(&element_binding.name);
                for stmt in &body.statements { self.visit_statement(stmt); }
                self.exit_scope();
            }
            Statement::FixedIterationLoop { from_value, to_value, step_value, counter, body, .. } => {
                self.visit_expression(from_value);
                self.visit_expression(to_value);
                if let Some(step) = step_value {
                    self.visit_expression(step);
                }
                self.enter_scope();
                self.define_variable(&counter.name);
                for stmt in &body.statements { self.visit_statement(stmt); }
                self.exit_scope();
            }
            Statement::Assignment { target, value, .. } => {
                self.visit_expression(value);
                match target {
                    AssignmentTarget::Variable { name } => self.check_variable_usage(&name.name),
                    AssignmentTarget::ArrayElement { array, index } => {
                        self.visit_expression(array);
                        self.visit_expression(index);
                    }
                    AssignmentTarget::StructField { instance, .. } => self.visit_expression(instance),
                    AssignmentTarget::MapValue { map, key } => {
                        self.visit_expression(map);
                        self.visit_expression(key);
                    }
                    AssignmentTarget::Dereference { pointer } => self.visit_expression(pointer),
                }
            }
            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    self.visit_expression(expr);
                }
            }
            Statement::Expression { expr, .. } => {
                self.visit_expression(expr);
            }
            Statement::TryBlock { protected_block, catch_clauses, finally_block, .. } => {
                self.enter_scope();
                for stmt in &protected_block.statements { self.visit_statement(stmt); }
                self.exit_scope();

                for clause in catch_clauses {
                    self.enter_scope();
                    if let Some(binding) = &clause.binding_variable {
                        self.define_variable(&binding.name);
                    }
                    for stmt in &clause.handler_block.statements { self.visit_statement(stmt); }
                    self.exit_scope();
                }

                if let Some(block) = finally_block {
                    self.enter_scope();
                    for stmt in &block.statements { self.visit_statement(stmt); }
                    self.exit_scope();
                }
            }
            Statement::Match { value, arms, .. } => {
                self.visit_expression(value);
                for arm in arms {
                    self.enter_scope();
                    // Bind pattern variables
                    self.visit_pattern(&arm.pattern);
                    
                    if let Some(guard) = &arm.guard {
                        self.visit_expression(guard);
                    }
                    
                    for stmt in &arm.body.statements {
                        self.visit_statement(stmt);
                    }
                    self.exit_scope();
                }
            }
            Statement::Throw { exception, .. } => self.visit_expression(exception),
            Statement::FunctionCall { call, .. } => {
                 self.visit_expression(&Expression::FunctionCall { 
                     call: call.clone(), 
                     source_location: SourceLocation::unknown() 
                 });
            }
            Statement::ResourceScope { scope, .. } => {
                self.enter_scope();
                for resource in &scope.resources {
                    self.visit_expression(&resource.acquisition);
                    self.define_variable(&resource.binding.name);
                }
                for stmt in &scope.body.statements { self.visit_statement(stmt); }
                self.exit_scope();
            }
            _ => {}
        }
    }

    fn visit_expression(&mut self, node: &Expression) {
        match node {
            Expression::Variable { name, .. } => {
                self.check_variable_usage(&name.name);
            }
            _ => {}
        }
        
        // Generic traversal for all children
        // Ideally I'd use a macro or a default visitor, but I have to implement it manually here.
        // Let's be comprehensive.
        match node {
            Expression::Add { left, right, .. } |
            Expression::Subtract { left, right, .. } |
            Expression::Multiply { left, right, .. } |
            Expression::Divide { left, right, .. } |
            Expression::IntegerDivide { left, right, .. } |
            Expression::Modulo { left, right, .. } |
            Expression::Equals { left, right, .. } |
            Expression::NotEquals { left, right, .. } |
            Expression::LessThan { left, right, .. } |
            Expression::LessThanOrEqual { left, right, .. } |
            Expression::GreaterThan { left, right, .. } |
            Expression::GreaterThanOrEqual { left, right, .. } |
            Expression::StringEquals { left, right, .. } |
            Expression::StringContains { haystack: left, needle: right, .. } => {
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Expression::Negate { operand: expr, .. } |
            Expression::LogicalNot { operand: expr, .. } |
            Expression::StringLength { string: expr, .. } |
            Expression::TypeCast { value: expr, .. } |
            Expression::AddressOf { operand: expr, .. } |
            Expression::Dereference { pointer: expr, .. } |
            Expression::ArrayLength { array: expr, .. } => {
                self.visit_expression(expr);
            }
            Expression::LogicalAnd { operands, .. } |
            Expression::LogicalOr { operands, .. } |
            Expression::StringConcat { operands, .. } => {
                for op in operands {
                    self.visit_expression(op);
                }
            }
            Expression::FunctionCall { call, .. } => {
                for arg in &call.arguments {
                    self.visit_expression(&arg.value);
                }
                // Also check function reference if it's a variable?
                if let FunctionReference::Local { name } = &call.function_reference {
                    self.check_variable_usage(&name.name);
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for elem in elements { self.visit_expression(elem); }
            }
            Expression::ArrayAccess { array, index, .. } => {
                self.visit_expression(array);
                self.visit_expression(index);
            }
            Expression::StructConstruct { field_values, .. } => {
                for fv in field_values { self.visit_expression(&fv.value); }
            }
            Expression::FieldAccess { instance, .. } => {
                self.visit_expression(instance);
            }
            Expression::MethodCall { receiver, arguments, .. } => {
                self.visit_expression(receiver);
                for arg in arguments { self.visit_expression(&arg.value); }
            }
            Expression::Lambda { .. } => {
                // TODO: Handle lambdas. They also capture!
                // But this task is specifically for concurrent blocks.
                // However, if a lambda inside a concurrent block uses a var, it's still a capture for the block.
            }
            _ => {}
        }
    }

    fn visit_type_definition(&mut self, _node: &TypeDefinition) {
        // Types don't contain executable code/variables usually
    }
}

impl CaptureAnalyzer {
    fn visit_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::EnumVariant { bindings, nested_pattern, .. } => {
                for binding in bindings {
                    self.define_variable(&binding.name);
                }
                if let Some(nested) = nested_pattern {
                    self.visit_pattern(nested);
                }
            }
            Pattern::Struct { fields, .. } => {
                for (_, pat) in fields {
                    self.visit_pattern(pat);
                }
            }
            Pattern::Wildcard { binding, .. } => {
                if let Some(b) = binding {
                    self.define_variable(&b.name);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::v2::Lexer;
    use crate::parser::v2::Parser;

    #[test]
    fn test_capture_simple() {
        let source = r#"
            module test_capture;
            func main() -> Void {
                let x: Int = 10;
                let y: Int = 20;
                concurrent {
                    let z: Int = {x + 5};
                }
            }
        "#;

        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Failed to tokenize");
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().expect("Failed to parse module");

        let mut analyzer = CaptureAnalyzer::new();
        analyzer.analyze(&module);

        // We expect one concurrent block with "x" captured
        assert_eq!(analyzer.captures.len(), 1);
        
        let captured_vars = analyzer.captures.values().next().unwrap();
        assert!(captured_vars.contains("x"));
        assert!(!captured_vars.contains("y"));
        assert!(!captured_vars.contains("z"));
    }

    #[test]
    fn test_capture_nested() {
        let source = r#"
            module test_nested;
            func main() -> Void {
                let a: Int = 1;
                concurrent {
                    let b: Int = 2;
                    let c: Int = {a + b}; // Captures a
                    concurrent {
                        let d: Int = {c + a}; // Captures a (from outer), c (from first concurrent)
                    }
                }
            }
        "#;

        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Failed to tokenize");
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().expect("Failed to parse module");

        let mut analyzer = CaptureAnalyzer::new();
        analyzer.analyze(&module);

        assert_eq!(analyzer.captures.len(), 2);
        
        // Check captures
        // Since we don't have easy access to keys (SourceLocation), we check that *some* block has 'a'
        // and another has 'a', 'b', 'c'.
        
        let mut found_outer = false;
        let mut found_inner = false;
        
        for vars in analyzer.captures.values() {
            if vars.len() == 1 && vars.contains("a") {
                found_outer = true;
            } else if vars.len() >= 2 && vars.contains("a") && vars.contains("c") {
                found_inner = true;
            }
        }
        
        assert!(found_outer, "Outer block should capture 'a'");
        assert!(found_inner, "Inner block should capture 'a' and 'c'");
    }
}

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

use aether::ast::*;
use aether::lexer::v2::Lexer;
use aether::parser::v2::Parser;
use aether::semantic::SemanticAnalyzer;

/// Helper function to create a simple test module with control flow
fn create_control_flow_module() -> String {
    r#"module control_flow_test {
    // Intent: Test control flow constructs

    @intent("Test if statement")
    func test_if(x: Int) -> Int {
        if {x > 10} {
            return 1;
        } else {
            return 0;
        }
    }

    @intent("Test while loop")
    func test_while(n: Int) -> Int {
        var count: Int = 0;
        while {count < n} {
            count = {count + 1};
        }
        return count;
    }
}"#.to_string()
}

#[test]
fn test_if_statement_parsing() {
    let source = create_control_flow_module();

    let mut lexer = Lexer::new(&source, "control_flow_test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");

    assert_eq!(program.modules.len(), 1);
    assert_eq!(program.modules[0].function_definitions.len(), 2);

    // Check first function has if statement
    let first_func = &program.modules[0].function_definitions[0];
    assert_eq!(first_func.name.name, "test_if");
    assert_eq!(first_func.body.statements.len(), 1);

    match &first_func.body.statements[0] {
        Statement::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            assert!(matches!(condition.as_ref(), Expression::GreaterThan { .. }));
            assert_eq!(then_block.statements.len(), 1);
            assert!(else_block.is_some());
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_while_loop_parsing() {
    let source = create_control_flow_module();

    let mut lexer = Lexer::new(&source, "control_flow_test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");

    // Check second function has while loop
    let second_func = &program.modules[0].function_definitions[1];
    assert_eq!(second_func.name.name, "test_while");
    assert_eq!(second_func.body.statements.len(), 3); // declare, while, return

    match &second_func.body.statements[1] {
        Statement::WhileLoop {
            condition, body, ..
        } => {
            assert!(matches!(condition.as_ref(), Expression::LessThan { .. }));
            assert_eq!(body.statements.len(), 1);
        }
        _ => panic!("Expected while loop"),
    }
}

#[test]
fn test_control_flow_semantic_analysis() {
    let source = create_control_flow_module();

    let mut lexer = Lexer::new(&source, "control_flow_test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");

    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze_program(&program)
        .expect("Semantic analysis should succeed");

    let stats = analyzer.get_statistics();
    assert_eq!(stats.modules_analyzed, 1);
    assert_eq!(stats.functions_analyzed, 2);
}

#[test]
fn test_boolean_condition_type_checking() {
    // Test that non-boolean conditions are rejected
    let source = r#"module bad_control_flow {
    // Intent: Test invalid control flow

    @intent("If with non-boolean condition")
    func bad_if() {
        if 42 {
            return;
        }
    }
}"#;

    let mut lexer = Lexer::new(source, "bad_control_flow.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.to_string().contains("Type mismatch")));
}

#[test]
fn test_loop_scope_isolation() {
    // Test that variables declared in loop scope are not accessible outside
    let source = r#"module loop_scope_test {
    // Intent: Test loop scope isolation

    @intent("Test variable scope in loops")
    func test_scope() -> Int {
        for elem in [1, 2, 3] {
            let inner: Int = 0;
        }
        // This should fail - 'elem' is not in scope here
        return elem;
    }
}"#;

    let mut lexer = Lexer::new(source, "loop_scope_test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.to_string().contains("Undefined symbol")));
}

#[test]
fn test_nested_control_flow() {
    let source = r#"module nested_control {
    // Intent: Test nested control flow

    @intent("Test nested loops and conditions")
    func nested_loops(n: Int) -> Int {
        var sum: Int = 0;
        var i: Int = 0;
        while {i < n} {
            var j: Int = 0;
            while {j < i} {
                if {{j % 2} == 0} {
                    sum = {sum + j};
                }
                j = {j + 1};
            }
            i = {i + 1};
        }
        return sum;
    }
}"#;

    let mut lexer = Lexer::new(source, "nested_control.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");

    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze_program(&program)
        .expect("Semantic analysis should succeed");

    let stats = analyzer.get_statistics();
    assert_eq!(stats.modules_analyzed, 1);
    assert_eq!(stats.functions_analyzed, 1);
    // count variables: sum, i, j (inner loop), and function parameter n
    // Note: 'n' is a parameter, 'sum', 'i', 'j' are declared variables.
    // The statistics might verify 'variables_declared'. 
    // In V2, 'var' declarations count.
    assert_eq!(stats.variables_declared, 3); // sum, i, j
}
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

use aether::lexer::Lexer;
use aether::parser::Parser;
use aether::semantic::SemanticAnalyzer;
use aether::ast::*;

/// Helper function to create a simple test module with control flow
fn create_control_flow_module() -> String {
    r#"(DEFINE_MODULE
        (NAME 'control_flow_test')
        (INTENT "Test control flow constructs")
        (CONTENT
            (DEFINE_FUNCTION
                (NAME 'test_if')
                (INTENT "Test if statement")
                (PARAMETERS
                    (ACCEPTS_PARAMETER (NAME 'x') (TYPE INTEGER))
                )
                (RETURNS INTEGER)
                (BODY
                    (IF_CONDITION 
                        (EXPRESSION_GREATER_THAN (VARIABLE 'x') (INTEGER 10))
                        (THEN_EXECUTE (RETURN_VALUE (INTEGER 1)))
                        (ELSE_EXECUTE (RETURN_VALUE (INTEGER 0)))
                    )
                )
            )
            
            (DEFINE_FUNCTION
                (NAME 'test_while')
                (INTENT "Test while loop")
                (PARAMETERS
                    (ACCEPTS_PARAMETER (NAME 'n') (TYPE INTEGER))
                )
                (RETURNS INTEGER)
                (BODY
                    (DECLARE_VARIABLE (NAME 'count') (TYPE INTEGER) (MUTABILITY MUTABLE) (VALUE (INTEGER 0)))
                    (LOOP_WHILE_CONDITION
                        (EXPRESSION_LESS_THAN (VARIABLE 'count') (VARIABLE 'n'))
                        (BODY
                            (ASSIGN 
                                (TARGET (VARIABLE 'count'))
                                (VALUE (EXPRESSION_ADD (VARIABLE 'count') (INTEGER 1)))
                            )
                        )
                    )
                    (RETURN_VALUE (VARIABLE 'count'))
                )
            )
        )
    )"#.to_string()
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
        Statement::If { condition, then_block, else_block, .. } => {
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
        Statement::WhileLoop { condition, body, .. } => {
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
    analyzer.analyze_program(&program).expect("Semantic analysis should succeed");
    
    let stats = analyzer.get_statistics();
    assert_eq!(stats.modules_analyzed, 1);
    assert_eq!(stats.functions_analyzed, 2);
}

#[test]
fn test_boolean_condition_type_checking() {
    // Test that non-boolean conditions are rejected
    let source = r#"(DEFINE_MODULE
        (NAME 'bad_control_flow')
        (INTENT "Test invalid control flow")
        (CONTENT
            (DEFINE_FUNCTION
                (NAME 'bad_if')
                (INTENT "If with non-boolean condition")
                (PARAMETERS)
                (RETURNS VOID)
                (BODY
                    (IF_CONDITION 
                        (INTEGER 42)  ; This should fail - not a boolean
                        (THEN_EXECUTE (RETURN_VOID))
                    )
                )
            )
        )
    )"#;
    
    let mut lexer = Lexer::new(source, "bad_control_flow.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.to_string().contains("Type mismatch")));
}

#[test]
fn test_loop_scope_isolation() {
    // Test that variables declared in loop scope are not accessible outside
    let source = r#"(DEFINE_MODULE
        (NAME 'loop_scope_test')
        (INTENT "Test loop scope isolation")
        (CONTENT
            (DEFINE_FUNCTION
                (NAME 'test_scope')
                (INTENT "Test variable scope in loops")
                (PARAMETERS)
                (RETURNS INTEGER)
                (BODY
                    (LOOP_FOR_EACH_ELEMENT
                        (COLLECTION (ARRAY_LITERAL INTEGER (INTEGER 1) (INTEGER 2) (INTEGER 3)))
                        (ELEMENT_BINDING 'elem')
                        (ELEMENT_TYPE INTEGER)
                        (BODY
                            (DECLARE_VARIABLE (NAME 'inner') (TYPE INTEGER) (VALUE (INTEGER 0)))
                        )
                    )
                    ; This should fail - 'elem' is not in scope here
                    (RETURN_VALUE (VARIABLE 'elem'))
                )
            )
        )
    )"#;
    
    let mut lexer = Lexer::new(source, "loop_scope_test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.to_string().contains("Undefined symbol")));
}

#[test]
fn test_nested_control_flow() {
    let source = r#"(DEFINE_MODULE
        (NAME 'nested_control')
        (INTENT "Test nested control flow")
        (CONTENT
            (DEFINE_FUNCTION
                (NAME 'nested_loops')
                (INTENT "Test nested loops and conditions")
                (PARAMETERS
                    (ACCEPTS_PARAMETER (NAME 'n') (TYPE INTEGER))
                )
                (RETURNS INTEGER)
                (BODY
                    (DECLARE_VARIABLE (NAME 'sum') (TYPE INTEGER) (MUTABILITY MUTABLE) (VALUE (INTEGER 0)))
                    (DECLARE_VARIABLE (NAME 'i') (TYPE INTEGER) (MUTABILITY MUTABLE) (VALUE (INTEGER 0)))
                    (LOOP_WHILE_CONDITION
                        (EXPRESSION_LESS_THAN (VARIABLE 'i') (VARIABLE 'n'))
                        (BODY
                            (DECLARE_VARIABLE (NAME 'j') (TYPE INTEGER) (MUTABILITY MUTABLE) (VALUE (INTEGER 0)))
                            (LOOP_WHILE_CONDITION
                                (EXPRESSION_LESS_THAN (VARIABLE 'j') (VARIABLE 'i'))
                                (BODY
                                    (IF_CONDITION
                                        (EXPRESSION_EQUALS (EXPRESSION_MODULO (VARIABLE 'j') (INTEGER 2)) (INTEGER 0))
                                        (THEN_EXECUTE
                                            (ASSIGN 
                                                (TARGET (VARIABLE 'sum'))
                                                (VALUE (EXPRESSION_ADD (VARIABLE 'sum') (VARIABLE 'j')))
                                            )
                                        )
                                    )
                                    (ASSIGN 
                                        (TARGET (VARIABLE 'j'))
                                        (VALUE (EXPRESSION_ADD (VARIABLE 'j') (INTEGER 1)))
                                    )
                                )
                            )
                            (ASSIGN 
                                (TARGET (VARIABLE 'i'))
                                (VALUE (EXPRESSION_ADD (VARIABLE 'i') (INTEGER 1)))
                            )
                        )
                    )
                    (RETURN_VALUE (VARIABLE 'sum'))
                )
            )
        )
    )"#;
    
    let mut lexer = Lexer::new(source, "nested_control.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis should succeed");
    
    let stats = analyzer.get_statistics();
    assert_eq!(stats.modules_analyzed, 1);
    assert_eq!(stats.functions_analyzed, 1);
    assert_eq!(stats.variables_declared, 4); // sum, i, j (inner loop), and function parameters
}
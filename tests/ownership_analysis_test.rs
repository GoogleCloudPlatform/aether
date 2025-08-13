#[cfg(test)]
mod ownership_analysis_tests {
    use aether::parser::Parser;
    use aether::lexer::Lexer;
    use aether::semantic::SemanticAnalyzer;
    use aether::error::SemanticError;
    
    #[test]
    fn test_use_after_move_detection() {
        let source = r#"
        module test {
            func consume(x: ^int) { }
            
            func main() -> int {
                let x: ^int = 42;
                consume(x);
                consume(x);  // Should fail: use after move
                return 0;
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Tokenization failed");
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().expect("Parsing failed");
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&ast);
        
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| matches!(e, SemanticError::UseAfterMove { variable, .. } if variable == "x")),
                    "Expected UseAfterMove error for variable 'x'");
        }
    }
    
    #[test]
    fn test_multiple_immutable_borrows_allowed() {
        let source = r#"
        module test {
            func borrow(x: &int) -> int {
                return *x;
            }
            
            func main() -> int {
                let x: ^int = 42;
                let r1 = borrow(&x);
                let r2 = borrow(&x);  // Should succeed: multiple immutable borrows
                let r3 = borrow(&x);  // Should succeed
                return r1 + r2 + r3;
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Tokenization failed");
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().expect("Parsing failed");
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&ast);
        
        assert!(result.is_ok(), "Multiple immutable borrows should be allowed");
    }
    
    #[test]
    fn test_cannot_move_while_borrowed() {
        let source = r#"
        module test {
            func consume(x: ^int) { }
            func borrow(x: &int) -> int { return *x; }
            
            func main() -> int {
                let x: ^int = 42;
                let r = &x;  // Borrow x
                consume(x);  // Should fail: cannot move while borrowed
                return *r;
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Tokenization failed");
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().expect("Parsing failed");
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&ast);
        
        assert!(result.is_err());
        // Check for appropriate error
    }
    
    #[test]
    fn test_string_ownership_transfer() {
        let source = r#"
        module test {
            func take_string(s: ^string) -> ^string {
                return s;
            }
            
            func main() -> int {
                let s1: ^string = "Hello";
                let s2: ^string = take_string(s1);  // Ownership transferred
                // Using s1 here would be an error
                return 0;
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Tokenization failed");
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().expect("Parsing failed");
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&ast);
        
        assert!(result.is_ok(), "String ownership transfer should succeed");
    }
    
    #[test]
    fn test_array_ownership_and_cleanup() {
        let source = r#"
        module test {
            func take_array(arr: ^[int; 5]) -> ^[int; 5] {
                return arr;
            }
            
            func main() -> int {
                let arr1: ^[int; 5] = [1, 2, 3, 4, 5];
                let arr2: ^[int; 5] = take_array(arr1);  // Ownership transferred
                return arr2[0];
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Tokenization failed");
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().expect("Parsing failed");
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&ast);
        
        assert!(result.is_ok(), "Array ownership transfer should succeed");
    }
    
    #[test]
    fn test_map_ownership() {
        let source = r#"
        module test {
            func take_map(m: ^map<string, int>) -> ^map<string, int> {
                return m;
            }
            
            func main() -> int {
                let mut m1: ^map<string, int> = {};
                m1["key"] = 42;
                let m2: ^map<string, int> = take_map(m1);  // Ownership transferred
                return m2["key"];
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Tokenization failed");
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().expect("Parsing failed");
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&ast);
        
        assert!(result.is_ok(), "Map ownership transfer should succeed");
    }
    
    #[test]
    fn test_shared_ownership_no_move() {
        let source = r#"
        module test {
            func use_shared(s: ~string) -> int {
                return 0;  // Would return string length
            }
            
            func main() -> int {
                let s1: ~string = ~"Shared string";
                let s2: ~string = s1;  // Ref count increased, not moved
                let len1 = use_shared(s1);  // Should work: s1 still valid
                let len2 = use_shared(s2);  // Should work: s2 also valid
                return len1 + len2;
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Tokenization failed");
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().expect("Parsing failed");
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&ast);
        
        assert!(result.is_ok(), "Shared ownership should not cause moves");
    }
    
    #[test]
    fn test_ownership_in_loops() {
        let source = r#"
        module test {
            func consume(x: ^int) { }
            
            func main() -> int {
                let mut i = 0;
                while i < 10 {
                    let x: ^int = i;
                    consume(x);  // x is consumed each iteration
                    i = i + 1;
                }
                return 0;
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Tokenization failed");
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().expect("Parsing failed");
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&ast);
        
        assert!(result.is_ok(), "Ownership in loops should work correctly");
    }
    
    #[test]
    fn test_ownership_type_checking() {
        let source = r#"
        module test {
            func expect_owned(x: ^int) { }
            func expect_borrowed(x: &int) { }
            
            func main() -> int {
                let x: ^int = 42;
                
                // This should fail: passing borrowed where owned is expected
                // expect_owned(&x);
                
                // This should succeed
                expect_borrowed(&x);
                
                // This should succeed (moves x)
                expect_owned(x);
                
                return 0;
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().expect("Tokenization failed");
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_program().expect("Parsing failed");
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&ast);
        
        assert!(result.is_ok(), "Ownership type checking should work");
    }
}
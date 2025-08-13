use aether::lexer::Lexer;
use aether::parser::Parser;
use std::fs;

fn main() {
    let content = fs::read_to_string("tests/test_error_recovery.aether").unwrap();
    let mut lexer = Lexer::new(&content, "tests/test_error_recovery.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    
    println!("Total tokens: {}", tokens.len());
    
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => {
            println!("Parse succeeded with {} modules", program.modules.len());
            if parser.has_errors() {
                println!("But found {} errors during parsing:", parser.get_errors().len());
                for error in parser.get_errors() {
                    println!("  - {}", error);
                }
            }
        }
        Err(e) => {
            println!("Parse failed: {}", e);
            if parser.has_errors() {
                println!("Additional errors:");
                for error in parser.get_errors() {
                    println!("  - {}", error);
                }
            }
        }
    }
}
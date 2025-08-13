use std::fs;

fn main() {
    let source = fs::read_to_string("test_simple_loop.aether").unwrap();
    let mut lexer = aether::lexer::Lexer::new(&source, "test_simple_loop.aether".to_string());
    let tokens = lexer.tokenize();
    
    match tokens {
        Ok(tokens) => {
            println!("Tokens: {:?}", tokens);
            let mut parser = aether::parser::Parser::new(tokens);
            let result = parser.parse_program();
            match result {
                Ok(program) => println!("Successfully parsed program with {} modules", program.modules.len()),
                Err(e) => println!("Parse error: {:?}", e),
            }
        }
        Err(e) => println!("Lexer error: {:?}", e),
    }
}
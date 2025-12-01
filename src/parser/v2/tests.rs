use super::*;
use crate::lexer::v2::Lexer;

/// Helper to create a parser from source code
fn parser_from_source(source: &str) -> Parser {
    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    Parser::new(tokens)
}

// ==================== BASIC PARSER TESTS ====================

#[test]
fn test_parser_new() {
    let parser = parser_from_source("let x");
    assert_eq!(parser.position, 0);
    assert!(parser.errors.is_empty());
}

#[test]
fn test_parser_peek() {
    let parser = parser_from_source("let x");
    assert!(matches!(
        parser.peek().token_type,
        TokenType::Keyword(Keyword::Let)
    ));
}

#[test]
fn test_parser_peek_next() {
    let parser = parser_from_source("let x");
    let next = parser.peek_next().unwrap();
    assert!(matches!(next.token_type, TokenType::Identifier(ref s) if s == "x"));
}

#[test]
fn test_parser_advance() {
    let mut parser = parser_from_source("let x");

    // First advance returns "let"
    let tok = parser.advance();
    assert!(matches!(tok.token_type, TokenType::Keyword(Keyword::Let)));

    // Now peek should be "x"
    assert!(matches!(parser.peek().token_type, TokenType::Identifier(ref s) if s == "x"));
}

#[test]
fn test_parser_previous() {
    let mut parser = parser_from_source("let x");
    parser.advance();

    let prev = parser.previous();
    assert!(matches!(prev.token_type, TokenType::Keyword(Keyword::Let)));
}

#[test]
fn test_parser_is_at_end() {
    let mut parser = parser_from_source("x");

    assert!(!parser.is_at_end());
    parser.advance(); // x
    assert!(parser.is_at_end()); // Now at EOF
}

#[test]
fn test_parser_check() {
    let parser = parser_from_source("let x = 42");

    assert!(parser.check(&TokenType::Keyword(Keyword::Let)));
    assert!(!parser.check(&TokenType::Identifier("x".to_string())));
}

#[test]
fn test_parser_check_keyword() {
    let parser = parser_from_source("func main");

    assert!(parser.check_keyword(Keyword::Func));
    assert!(!parser.check_keyword(Keyword::Let));
}

#[test]
fn test_parser_expect_success() {
    let mut parser = parser_from_source("let x");

    let result = parser.expect(&TokenType::Keyword(Keyword::Let), "expected 'let'");
    assert!(result.is_ok());

    // Position should have advanced
    assert!(matches!(parser.peek().token_type, TokenType::Identifier(ref s) if s == "x"));
}

#[test]
fn test_parser_expect_failure() {
    let mut parser = parser_from_source("let x");

    let result = parser.expect(&TokenType::Keyword(Keyword::Func), "expected 'func'");
    assert!(result.is_err());

    // Position should NOT have advanced
    assert!(matches!(
        parser.peek().token_type,
        TokenType::Keyword(Keyword::Let)
    ));
}

#[test]
fn test_parser_expect_keyword_success() {
    let mut parser = parser_from_source("module Test");

    let result = parser.expect_keyword(Keyword::Module, "expected 'module'");
    assert!(result.is_ok());
}

#[test]
fn test_parser_expect_keyword_failure() {
    let mut parser = parser_from_source("module Test");

    let result = parser.expect_keyword(Keyword::Func, "expected 'func'");
    assert!(result.is_err());
}

#[test]
fn test_parser_match_any() {
    let mut parser = parser_from_source("+ - *");

    // Should match Plus
    assert!(parser.match_any(&[TokenType::Plus, TokenType::Minus]));

    // Now at Minus, should match
    assert!(parser.match_any(&[TokenType::Plus, TokenType::Minus]));

    // Now at Star, should NOT match
    assert!(!parser.match_any(&[TokenType::Plus, TokenType::Minus]));
}

#[test]
fn test_parser_current_location() {
    let parser = parser_from_source("let x");
    let loc = parser.current_location();

    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 1);
}

#[test]
fn test_parser_add_error() {
    let mut parser = parser_from_source("let x");

    assert!(parser.errors().is_empty());

    parser.add_error(ParserError::UnexpectedToken {
        expected: "test".to_string(),
        found: "other".to_string(),
        location: parser.current_location(),
    });

    assert_eq!(parser.errors().len(), 1);
}

#[test]
fn test_parser_empty_input() {
    let parser = parser_from_source("");

    // Should just have EOF
    assert!(parser.is_at_end());
}

#[test]
fn test_parser_multiline_input() {
    let mut parser = parser_from_source("let\nx\n=\n42");

    // Should be able to parse through newlines
    parser.expect_keyword(Keyword::Let, "let").unwrap();

    // Next should be identifier
    assert!(matches!(parser.peek().token_type, TokenType::Identifier(ref s) if s == "x"));
}

#[test]
fn test_parser_with_comments() {
    let parser = parser_from_source("// comment\nlet x");

    // Comments should be skipped, first token is 'let'
    assert!(parser.check_keyword(Keyword::Let));
}

// ==================== MODULE PARSING TESTS ====================

#[test]
fn test_parse_empty_module() {
    let mut parser = parser_from_source("module Test { }");
    let result = parser.parse_module();

    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.name.name, "Test");
    assert!(module.imports.is_empty());
    assert!(module.function_definitions.is_empty());
}

#[test]
fn test_parse_module_with_single_import() {
    let mut parser = parser_from_source("module Test { import std; }");
    let result = parser.parse_module();

    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.name.name, "Test");
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module_name.name, "std");
}

#[test]
fn test_parse_module_with_dotted_import() {
    let mut parser = parser_from_source("module Test { import std.io; }");
    let result = parser.parse_module();

    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module_name.name, "std.io");
}

#[test]
fn test_parse_module_with_multiple_imports() {
    let mut parser = parser_from_source(
        "module Test { import std.io; import std.collections; import math; }",
    );
    let result = parser.parse_module();

    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.imports.len(), 3);
    assert_eq!(module.imports[0].module_name.name, "std.io");
    assert_eq!(module.imports[1].module_name.name, "std.collections");
    assert_eq!(module.imports[2].module_name.name, "math");
}

#[test]
fn test_parse_module_with_deeply_nested_import() {
    let mut parser =
        parser_from_source("module Test { import std.collections.hashmap.HashMap; }");
    let result = parser.parse_module();

    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(
        module.imports[0].module_name.name,
        "std.collections.hashmap.HashMap"
    );
}

#[test]
fn test_parse_module_error_missing_name() {
    let mut parser = parser_from_source("module { }");
    let result = parser.parse_module();

    assert!(result.is_err());
}

#[test]
fn test_parse_module_error_missing_open_brace() {
    let mut parser = parser_from_source("module Test }");
    let result = parser.parse_module();

    assert!(result.is_err());
}

#[test]
fn test_parse_module_error_missing_close_brace() {
    let mut parser = parser_from_source("module Test {");
    let result = parser.parse_module();

    assert!(result.is_err());
}

#[test]
fn test_parse_import_error_missing_semicolon() {
    let mut parser = parser_from_source("module Test { import std }");
    let result = parser.parse_module();

    assert!(result.is_err());
}

#[test]
fn test_parse_module_multiline() {
    let source = r#"
module MyModule {
import std.io;
import std.collections;
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_module();

    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.name.name, "MyModule");
    assert_eq!(module.imports.len(), 2);
}

#[test]
fn test_parse_module_with_comments() {
    let source = r#"
// Module documentation
module Test {
// Import the standard library
import std;
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_module();

    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.name.name, "Test");
    assert_eq!(module.imports.len(), 1);
}

// ==================== TYPE PARSING TESTS ====================

#[test]
fn test_parse_type_int() {
    let mut parser = parser_from_source("Int");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    assert!(matches!(
        type_spec,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            ..
        }
    ));
}

#[test]
fn test_parse_type_int64() {
    let mut parser = parser_from_source("Int64");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    assert!(matches!(
        type_spec,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer64,
            ..
        }
    ));
}

#[test]
fn test_parse_type_float() {
    let mut parser = parser_from_source("Float");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    assert!(matches!(
        type_spec,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Float,
            ..
        }
    ));
}

#[test]
fn test_parse_type_string() {
    let mut parser = parser_from_source("String");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    assert!(matches!(
        type_spec,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::String,
            ..
        }
    ));
}

#[test]
fn test_parse_type_bool() {
    let mut parser = parser_from_source("Bool");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    assert!(matches!(
        type_spec,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Boolean,
            ..
        }
    ));
}

#[test]
fn test_parse_type_void() {
    let mut parser = parser_from_source("Void");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    assert!(matches!(
        type_spec,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            ..
        }
    ));
}

#[test]
fn test_parse_type_sizet() {
    let mut parser = parser_from_source("SizeT");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    assert!(matches!(
        type_spec,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::SizeT,
            ..
        }
    ));
}

#[test]
fn test_parse_type_array() {
    let mut parser = parser_from_source("Array<Int>");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Array { element_type, .. } = type_spec {
        assert!(matches!(
            *element_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                ..
            }
        ));
    } else {
        panic!("Expected Array type");
    }
}

#[test]
fn test_parse_type_nested_array() {
    let mut parser = parser_from_source("Array<Array<Int>>");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Array { element_type, .. } = type_spec {
        assert!(matches!(*element_type, TypeSpecifier::Array { .. }));
    } else {
        panic!("Expected nested Array type");
    }
}

#[test]
fn test_parse_type_map() {
    let mut parser = parser_from_source("Map<String, Int>");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Map {
        key_type,
        value_type,
        ..
    } = type_spec
    {
        assert!(matches!(
            *key_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::String,
                ..
            }
        ));
        assert!(matches!(
            *value_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                ..
            }
        ));
    } else {
        panic!("Expected Map type");
    }
}

#[test]
fn test_parse_type_pointer() {
    let mut parser = parser_from_source("Pointer<Int>");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Pointer {
        target_type,
        is_mutable,
        ..
    } = type_spec
    {
        assert!(!is_mutable);
        assert!(matches!(
            *target_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                ..
            }
        ));
    } else {
        panic!("Expected Pointer type");
    }
}

#[test]
fn test_parse_type_mut_pointer() {
    let mut parser = parser_from_source("MutPointer<Void>");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Pointer {
        target_type,
        is_mutable,
        ..
    } = type_spec
    {
        assert!(is_mutable);
        assert!(matches!(
            *target_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                ..
            }
        ));
    } else {
        panic!("Expected MutPointer type");
    }
}

#[test]
fn test_parse_type_owned() {
    let mut parser = parser_from_source("^String");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Owned {
        ownership,
        base_type,
        ..
    } = type_spec
    {
        assert_eq!(ownership, OwnershipKind::Owned);
        assert!(matches!(
            *base_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::String,
                ..
            }
        ));
    } else {
        panic!("Expected Owned type");
    }
}

#[test]
fn test_parse_type_borrowed() {
    let mut parser = parser_from_source("&Int");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Owned {
        ownership,
        base_type,
        ..
    } = type_spec
    {
        assert_eq!(ownership, OwnershipKind::Borrowed);
        assert!(matches!(
            *base_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                ..
            }
        ));
    } else {
        panic!("Expected Borrowed type");
    }
}

#[test]
fn test_parse_type_borrowed_mut() {
    let mut parser = parser_from_source("&mut Int");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Owned {
        ownership,
        base_type,
        ..
    } = type_spec
    {
        assert_eq!(ownership, OwnershipKind::BorrowedMut);
        assert!(matches!(
            *base_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                ..
            }
        ));
    } else {
        panic!("Expected BorrowedMut type");
    }
}

#[test]
fn test_parse_type_shared() {
    let mut parser = parser_from_source("~Resource");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Owned {
        ownership,
        base_type,
        ..
    } = type_spec
    {
        assert_eq!(ownership, OwnershipKind::Shared);
        // Resource is a user-defined type
        assert!(matches!(*base_type, TypeSpecifier::Named { .. }));
    } else {
        panic!("Expected Shared type");
    }
}

#[test]
fn test_parse_type_named() {
    let mut parser = parser_from_source("MyCustomType");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Named { name, .. } = type_spec {
        assert_eq!(name.name, "MyCustomType");
    } else {
        panic!("Expected Named type");
    }
}

#[test]
fn test_parse_type_generic_named() {
    let mut parser = parser_from_source("Result<Int, String>");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Generic {
        base_type,
        type_arguments,
        ..
    } = type_spec
    {
        assert_eq!(base_type.name, "Result");
        assert_eq!(type_arguments.len(), 2);
    } else {
        panic!("Expected Generic type");
    }
}

#[test]
fn test_parse_type_complex_ownership() {
    let mut parser = parser_from_source("^Array<&Int>");
    let result = parser.parse_type();

    assert!(result.is_ok());
    let type_spec = result.unwrap();
    if let TypeSpecifier::Owned {
        ownership,
        base_type,
        ..
    } = type_spec
    {
        assert_eq!(ownership, OwnershipKind::Owned);
        if let TypeSpecifier::Array { element_type, .. } = *base_type {
            if let TypeSpecifier::Owned {
                ownership: inner_ownership,
                ..
            } = *element_type
            {
                assert_eq!(inner_ownership, OwnershipKind::Borrowed);
            } else {
                panic!("Expected borrowed element type");
            }
        } else {
            panic!("Expected Array base type");
        }
    } else {
        panic!("Expected Owned type");
    }
}

#[test]
fn test_parse_type_error_missing_generic_close() {
    let mut parser = parser_from_source("Array<Int");
    let result = parser.parse_type();

    assert!(result.is_err());
}

#[test]
fn test_parse_type_error_missing_map_comma() {
    let mut parser = parser_from_source("Map<String Int>");
    let result = parser.parse_type();

    assert!(result.is_err());
}

// ==================== FUNCTION PARSING TESTS ====================

#[test]
fn test_parse_function_no_params_no_return() {
    let mut parser = parser_from_source("func foo() { }");
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.name.name, "foo");
    assert!(func.parameters.is_empty());
    // Default return type should be Void
    assert!(matches!(
        *func.return_type,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            ..
        }
    ));
}

#[test]
fn test_parse_function_with_return_type() {
    let mut parser = parser_from_source("func answer() -> Int { }");
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.name.name, "answer");
    assert!(func.parameters.is_empty());
    assert!(matches!(
        *func.return_type,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            ..
        }
    ));
}

#[test]
fn test_parse_function_single_param() {
    let mut parser = parser_from_source("func greet(name: String) { }");
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.name.name, "greet");
    assert_eq!(func.parameters.len(), 1);
    assert_eq!(func.parameters[0].name.name, "name");
    assert!(matches!(
        *func.parameters[0].param_type,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::String,
            ..
        }
    ));
}

#[test]
fn test_parse_function_multiple_params() {
    let mut parser = parser_from_source("func add(a: Int, b: Int) -> Int { }");
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.name.name, "add");
    assert_eq!(func.parameters.len(), 2);
    assert_eq!(func.parameters[0].name.name, "a");
    assert_eq!(func.parameters[1].name.name, "b");
    assert!(matches!(
        *func.return_type,
        TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            ..
        }
    ));
}

#[test]
fn test_parse_function_complex_types() {
    let mut parser = parser_from_source(
        "func process(items: Array<Int>, config: Map<String, Int>) -> Bool { }",
    );
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.name.name, "process");
    assert_eq!(func.parameters.len(), 2);
    assert!(matches!(
        *func.parameters[0].param_type,
        TypeSpecifier::Array { .. }
    ));
    assert!(matches!(
        *func.parameters[1].param_type,
        TypeSpecifier::Map { .. }
    ));
}

#[test]
fn test_parse_function_ownership_types() {
    let mut parser =
        parser_from_source("func transfer(owned: ^String, borrowed: &Int) -> Void { }");
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.parameters.len(), 2);
    assert!(matches!(
        *func.parameters[0].param_type,
        TypeSpecifier::Owned {
            ownership: OwnershipKind::Owned,
            ..
        }
    ));
    assert!(matches!(
        *func.parameters[1].param_type,
        TypeSpecifier::Owned {
            ownership: OwnershipKind::Borrowed,
            ..
        }
    ));
}

#[test]
fn test_parse_function_with_body_content() {
    // Body content is skipped for now, but structure should parse
    let mut parser = parser_from_source("func main() { let x = 42; return x; }");
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.name.name, "main");
}

#[test]
fn test_parse_function_multiline() {
    let source = r#"
func calculate(
a: Int,
b: Int,
c: Int
) -> Int {
// body
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.name.name, "calculate");
    assert_eq!(func.parameters.len(), 3);
}

#[test]
fn test_parse_function_error_missing_name() {
    let mut parser = parser_from_source("func () { }");
    let result = parser.parse_function();

    assert!(result.is_err());
}

#[test]
fn test_parse_function_error_missing_open_paren() {
    let mut parser = parser_from_source("func foo) { }");
    let result = parser.parse_function();

    assert!(result.is_err());
}

#[test]
fn test_parse_function_error_missing_close_paren() {
    let mut parser = parser_from_source("func foo( { }");
    let result = parser.parse_function();

    assert!(result.is_err());
}

#[test]
fn test_parse_function_error_missing_body() {
    let mut parser = parser_from_source("func foo()");
    let result = parser.parse_function();

    assert!(result.is_err());
}

#[test]
fn test_parse_function_error_missing_param_type() {
    let mut parser = parser_from_source("func foo(x) { }");
    let result = parser.parse_function();

    assert!(result.is_err());
}

#[test]
fn test_parse_function_pointer_return() {
    let mut parser = parser_from_source("func allocate(size: SizeT) -> Pointer<Void> { }");
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert!(matches!(
        *func.return_type,
        TypeSpecifier::Pointer {
            is_mutable: false,
            ..
        }
    ));
}

// ==================== Annotation Parsing Tests ====================

#[test]
fn test_parse_annotation_simple() {
    let mut parser = parser_from_source("@test");
    let result = parser.parse_annotation();

    assert!(result.is_ok());
    let annotation = result.unwrap();
    assert_eq!(annotation.name, "test");
    assert!(annotation.arguments.is_empty());
}

#[test]
fn test_parse_annotation_with_labeled_string_arg() {
    let mut parser = parser_from_source("@extern(library: \"libc\")");
    let result = parser.parse_annotation();

    assert!(result.is_ok());
    let annotation = result.unwrap();
    assert_eq!(annotation.name, "extern");
    assert_eq!(annotation.arguments.len(), 1);
    assert_eq!(annotation.arguments[0].label, Some("library".to_string()));
    assert!(
        matches!(&annotation.arguments[0].value, AnnotationValue::String(s) if s == "libc")
    );
}

#[test]
fn test_parse_annotation_with_multiple_args() {
    let mut parser = parser_from_source("@extern(library: \"libc\", symbol: \"malloc\")");
    let result = parser.parse_annotation();

    assert!(result.is_ok());
    let annotation = result.unwrap();
    assert_eq!(annotation.name, "extern");
    assert_eq!(annotation.arguments.len(), 2);
    assert_eq!(annotation.arguments[0].label, Some("library".to_string()));
    assert_eq!(annotation.arguments[1].label, Some("symbol".to_string()));
    assert!(
        matches!(&annotation.arguments[1].value, AnnotationValue::String(s) if s == "malloc")
    );
}

#[test]
fn test_parse_annotation_with_braced_expression() {
    let mut parser = parser_from_source("@requires({n > 0})");
    let result = parser.parse_annotation();

    assert!(result.is_ok());
    let annotation = result.unwrap();
    assert_eq!(annotation.name, "requires");
    assert_eq!(annotation.arguments.len(), 1);
    assert!(annotation.arguments[0].label.is_none());
    
    match &annotation.arguments[0].value {
        AnnotationValue::Expression(expr) => {
            match &**expr {
                Expression::GreaterThan { left, right, .. } => {
                    match &**left {
                        Expression::Variable { name, .. } => assert_eq!(name.name, "n"),
                        _ => panic!("Expected variable left"),
                    }
                    match &**right {
                        Expression::IntegerLiteral { value, .. } => assert_eq!(*value, 0),
                        _ => panic!("Expected integer right"),
                    }
                }
                _ => panic!("Expected GreaterThan expression, found {:?}", expr),
            }
        }
        _ => panic!("Expected Expression value"),
    }
}

#[test]
fn test_parse_annotation_with_identifier_value() {
    let mut parser = parser_from_source("@category(math)");
    let result = parser.parse_annotation();

    assert!(result.is_ok());
    let annotation = result.unwrap();
    assert_eq!(annotation.name, "category");
    assert_eq!(annotation.arguments.len(), 1);
    assert!(
        matches!(&annotation.arguments[0].value, AnnotationValue::Identifier(s) if s == "math")
    );
}

#[test]
fn test_parse_annotation_with_integer_value() {
    let mut parser = parser_from_source("@priority(10)");
    let result = parser.parse_annotation();

    assert!(result.is_ok());
    let annotation = result.unwrap();
    assert_eq!(annotation.name, "priority");
    assert_eq!(annotation.arguments.len(), 1);
    assert!(matches!(
        &annotation.arguments[0].value,
        AnnotationValue::Integer(10)
    ));
}

#[test]
fn test_parse_annotation_with_boolean_value() {
    let mut parser = parser_from_source("@deprecated(true)");
    let result = parser.parse_annotation();

    assert!(result.is_ok());
    let annotation = result.unwrap();
    assert_eq!(annotation.name, "deprecated");
    assert_eq!(annotation.arguments.len(), 1);
    assert!(matches!(
        &annotation.arguments[0].value,
        AnnotationValue::Boolean(true)
    ));
}

#[test]
fn test_parse_annotation_empty_parens() {
    let mut parser = parser_from_source("@marker()");
    let result = parser.parse_annotation();

    assert!(result.is_ok());
    let annotation = result.unwrap();
    assert_eq!(annotation.name, "marker");
    assert!(annotation.arguments.is_empty());
}

#[test]
fn test_parse_annotation_error_missing_name() {
    let mut parser = parser_from_source("@()");
    let result = parser.parse_annotation();

    assert!(result.is_err());
}

#[test]
fn test_parse_annotation_error_missing_close_paren() {
    let mut parser = parser_from_source("@test(x: 1");
    let result = parser.parse_annotation();

    assert!(result.is_err());
}

#[test]
fn test_parse_external_function_basic() {
    let mut parser = parser_from_source(
        "@extern(library: \"libc\") func malloc(size: SizeT) -> Pointer<Void>;",
    );

    // First parse the annotation
    let annotation = parser.parse_annotation().unwrap();
    assert_eq!(annotation.name, "extern");

    // Then parse the external function
    let result = parser.parse_external_function(annotation);

    assert!(result.is_ok());
    let ext_func = result.unwrap();
    assert_eq!(ext_func.name.name, "malloc");
    assert_eq!(ext_func.library, "libc");
    assert_eq!(ext_func.parameters.len(), 1);
    assert_eq!(ext_func.parameters[0].name.name, "size");
}

#[test]
fn test_parse_external_function_with_symbol() {
    let mut parser = parser_from_source("@extern(library: \"c\", symbol: \"_malloc\") func malloc(size: SizeT) -> Pointer<Void>;");

    let annotation = parser.parse_annotation().unwrap();
    let result = parser.parse_external_function(annotation);

    assert!(result.is_ok());
    let ext_func = result.unwrap();
    assert_eq!(ext_func.name.name, "malloc");
    assert_eq!(ext_func.library, "c");
    assert_eq!(ext_func.symbol.as_deref(), Some("_malloc"));
}

#[test]
fn test_parse_external_function_multiple_params() {
    let mut parser = parser_from_source("@extern(library: \"libc\") func memcpy(dest: Pointer<Void>, src: Pointer<Void>, n: SizeT) -> Pointer<Void>;");

    let annotation = parser.parse_annotation().unwrap();
    let result = parser.parse_external_function(annotation);

    assert!(result.is_ok());
    let ext_func = result.unwrap();
    assert_eq!(ext_func.name.name, "memcpy");
    assert_eq!(ext_func.parameters.len(), 3);
}

#[test]
fn test_parse_external_function_error_missing_semicolon() {
    let mut parser = parser_from_source(
        "@extern(library: \"libc\") func malloc(size: SizeT) -> Pointer<Void>",
    );

    let annotation = parser.parse_annotation().unwrap();
    let result = parser.parse_external_function(annotation);

    assert!(result.is_err());
}

// ==================== Variable Declaration Tests ====================

#[test]
fn test_parse_variable_declaration_with_type_and_value() {
    let mut parser = parser_from_source("let x: Int = 42;");
    let result = parser.parse_variable_declaration();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::VariableDeclaration {
        name,
        type_spec,
        mutability,
        initial_value,
        ..
    } = stmt
    {
        assert_eq!(name.name, "x");
        assert!(matches!(
            *type_spec,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                ..
            }
        ));
        assert!(matches!(mutability, Mutability::Immutable));
        assert!(initial_value.is_some());
        let value = initial_value.unwrap();
        assert!(matches!(
            *value,
            Expression::IntegerLiteral { value: 42, .. }
        ));
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn test_parse_variable_declaration_mutable() {
    let mut parser = parser_from_source("let mut counter: Int = 0;");
    let result = parser.parse_variable_declaration();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::VariableDeclaration {
        name, mutability, ..
    } = stmt
    {
        assert_eq!(name.name, "counter");
        assert!(matches!(mutability, Mutability::Mutable));
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn test_parse_variable_declaration_string_value() {
    let mut parser = parser_from_source("let name: String = \"hello\";");
    let result = parser.parse_variable_declaration();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::VariableDeclaration { initial_value, .. } = stmt {
        assert!(initial_value.is_some());
        let value = initial_value.unwrap();
        assert!(
            matches!(*value, Expression::StringLiteral { ref value, .. } if value == "hello")
        );
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn test_parse_variable_declaration_float_value() {
    let mut parser = parser_from_source("let pi: Float = 3.14;");
    let result = parser.parse_variable_declaration();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::VariableDeclaration { initial_value, .. } = stmt {
        assert!(initial_value.is_some());
        let value = initial_value.unwrap();
        assert!(matches!(*value, Expression::FloatLiteral { .. }));
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn test_parse_variable_declaration_bool_value() {
    let mut parser = parser_from_source("let flag: Bool = true;");
    let result = parser.parse_variable_declaration();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::VariableDeclaration { initial_value, .. } = stmt {
        assert!(initial_value.is_some());
        let value = initial_value.unwrap();
        assert!(matches!(
            *value,
            Expression::BooleanLiteral { value: true, .. }
        ));
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn test_parse_variable_declaration_char_value() {
    let mut parser = parser_from_source("let ch: Char = 'a';");
    let result = parser.parse_variable_declaration();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::VariableDeclaration { initial_value, .. } = stmt {
        assert!(initial_value.is_some());
        let value = initial_value.unwrap();
        assert!(matches!(
            *value,
            Expression::CharacterLiteral { value: 'a', .. }
        ));
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn test_parse_variable_declaration_no_initializer() {
    let mut parser = parser_from_source("let x: Int;");
    let result = parser.parse_variable_declaration();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::VariableDeclaration {
        name,
        initial_value,
        ..
    } = stmt
    {
        assert_eq!(name.name, "x");
        assert!(initial_value.is_none());
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn test_parse_variable_declaration_type_inference() {
    let mut parser = parser_from_source("let x = 42;");
    let result = parser.parse_variable_declaration();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::VariableDeclaration {
        name,
        type_spec,
        initial_value,
        ..
    } = stmt
    {
        assert_eq!(name.name, "x");
        // Type should be _inferred placeholder
        assert!(
            matches!(*type_spec, TypeSpecifier::Named { ref name, .. } if name.name == "_inferred")
        );
        assert!(initial_value.is_some());
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn test_parse_variable_declaration_variable_reference() {
    let mut parser = parser_from_source("let y: Int = x;");
    let result = parser.parse_variable_declaration();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::VariableDeclaration { initial_value, .. } = stmt {
        assert!(initial_value.is_some());
        let value = initial_value.unwrap();
        assert!(matches!(*value, Expression::Variable { ref name, .. } if name.name == "x"));
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn test_parse_variable_declaration_error_missing_semicolon() {
    let mut parser = parser_from_source("let x: Int = 42");
    let result = parser.parse_variable_declaration();

    assert!(result.is_err());
}

#[test]
fn test_parse_variable_declaration_error_missing_name() {
    let mut parser = parser_from_source("let : Int = 42;");
    let result = parser.parse_variable_declaration();

    assert!(result.is_err());
}

// ==================== Expression Parsing Tests ====================

#[test]
fn test_parse_expression_integer() {
    let mut parser = parser_from_source("42");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::IntegerLiteral { value: 42, .. }));
}

#[test]
fn test_parse_expression_float() {
    let mut parser = parser_from_source("3.14");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::FloatLiteral { .. }));
}

#[test]
fn test_parse_expression_string() {
    let mut parser = parser_from_source("\"hello world\"");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(
        matches!(expr, Expression::StringLiteral { ref value, .. } if value == "hello world")
    );
}

#[test]
fn test_parse_expression_boolean() {
    let mut parser = parser_from_source("true");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(
        expr,
        Expression::BooleanLiteral { value: true, .. }
    ));
}

#[test]
fn test_parse_expression_identifier() {
    let mut parser = parser_from_source("myVar");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Variable { ref name, .. } if name.name == "myVar"));
}

// ==================== Assignment Parsing Tests ====================

#[test]
fn test_parse_assignment_simple() {
    let mut parser = parser_from_source("x = 42;");
    let result = parser.parse_assignment();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Assignment { target, value, .. } = stmt {
        assert!(matches!(target, AssignmentTarget::Variable { ref name } if name.name == "x"));
        assert!(matches!(
            *value,
            Expression::IntegerLiteral { value: 42, .. }
        ));
    } else {
        panic!("Expected Assignment");
    }
}

#[test]
fn test_parse_assignment_string_value() {
    let mut parser = parser_from_source("name = \"hello\";");
    let result = parser.parse_assignment();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Assignment { target, value, .. } = stmt {
        assert!(
            matches!(target, AssignmentTarget::Variable { ref name } if name.name == "name")
        );
        assert!(
            matches!(*value, Expression::StringLiteral { ref value, .. } if value == "hello")
        );
    } else {
        panic!("Expected Assignment");
    }
}

#[test]
fn test_parse_assignment_variable_value() {
    let mut parser = parser_from_source("y = x;");
    let result = parser.parse_assignment();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Assignment { value, .. } = stmt {
        assert!(matches!(*value, Expression::Variable { ref name, .. } if name.name == "x"));
    } else {
        panic!("Expected Assignment");
    }
}

#[test]
fn test_parse_assignment_array_element() {
    let mut parser = parser_from_source("arr[0] = 42;");
    let result = parser.parse_assignment();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Assignment { target, .. } = stmt {
        if let AssignmentTarget::ArrayElement { array, index } = target {
            assert!(
                matches!(*array, Expression::Variable { ref name, .. } if name.name == "arr")
            );
            assert!(matches!(
                *index,
                Expression::IntegerLiteral { value: 0, .. }
            ));
        } else {
            panic!("Expected ArrayElement target");
        }
    } else {
        panic!("Expected Assignment");
    }
}

#[test]
fn test_parse_assignment_struct_field() {
    let mut parser = parser_from_source("point.x = 10;");
    let result = parser.parse_assignment();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Assignment { target, .. } = stmt {
        if let AssignmentTarget::StructField {
            instance,
            field_name,
        } = target
        {
            assert!(
                matches!(*instance, Expression::Variable { ref name, .. } if name.name == "point")
            );
            assert_eq!(field_name.name, "x");
        } else {
            panic!("Expected StructField target");
        }
    } else {
        panic!("Expected Assignment");
    }
}

#[test]
fn test_parse_assignment_error_missing_equals() {
    let mut parser = parser_from_source("x 42;");
    let result = parser.parse_assignment();

    assert!(result.is_err());
}

#[test]
fn test_parse_assignment_error_missing_semicolon() {
    let mut parser = parser_from_source("x = 42");
    let result = parser.parse_assignment();

    assert!(result.is_err());
}

#[test]
fn test_parse_assignment_error_missing_value() {
    let mut parser = parser_from_source("x = ;");
    let result = parser.parse_assignment();

    assert!(result.is_err());
}

// ==================== Binary Expression Parsing Tests ====================

#[test]
fn test_parse_binary_add() {
    let mut parser = parser_from_source("{1 + 2}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Add { left, right, .. } = expr {
        assert!(matches!(*left, Expression::IntegerLiteral { value: 1, .. }));
        assert!(matches!(
            *right,
            Expression::IntegerLiteral { value: 2, .. }
        ));
    } else {
        panic!("Expected Add expression, got {:?}", expr);
    }
}

#[test]
fn test_parse_binary_subtract() {
    let mut parser = parser_from_source("{10 - 5}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Subtract { .. }));
}

#[test]
fn test_parse_binary_multiply() {
    let mut parser = parser_from_source("{3 * 4}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Multiply { .. }));
}

#[test]
fn test_parse_binary_divide() {
    let mut parser = parser_from_source("{10 / 2}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Divide { .. }));
}

#[test]
fn test_parse_binary_modulo() {
    let mut parser = parser_from_source("{7 % 3}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Modulo { .. }));
}

#[test]
fn test_parse_binary_equals() {
    let mut parser = parser_from_source("{x == y}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Equals { .. }));
}

#[test]
fn test_parse_binary_not_equals() {
    let mut parser = parser_from_source("{a != b}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::NotEquals { .. }));
}

#[test]
fn test_parse_binary_less_than() {
    let mut parser = parser_from_source("{x < 10}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::LessThan { .. }));
}

#[test]
fn test_parse_binary_less_equal() {
    let mut parser = parser_from_source("{x <= 10}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::LessThanOrEqual { .. }));
}

#[test]
fn test_parse_binary_greater_than() {
    let mut parser = parser_from_source("{x > 0}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::GreaterThan { .. }));
}

#[test]
fn test_parse_binary_greater_equal() {
    let mut parser = parser_from_source("{x >= 0}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::GreaterThanOrEqual { .. }));
}

#[test]
fn test_parse_binary_and() {
    let mut parser = parser_from_source("{a && b}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::LogicalAnd { .. }));
}

#[test]
fn test_parse_binary_or() {
    let mut parser = parser_from_source("{a || b}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::LogicalOr { .. }));
}

#[test]
fn test_parse_binary_nested() {
    // {1 + {2 * 3}} - nested binary expressions
    let mut parser = parser_from_source("{1 + {2 * 3}}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Add { left, right, .. } = expr {
        assert!(matches!(*left, Expression::IntegerLiteral { value: 1, .. }));
        assert!(matches!(*right, Expression::Multiply { .. }));
    } else {
        panic!("Expected Add with nested Multiply");
    }
}

#[test]
fn test_parse_binary_with_variables() {
    let mut parser = parser_from_source("{x + y}");
    let result = parser.parse_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Add { left, right, .. } = expr {
        assert!(matches!(*left, Expression::Variable { ref name, .. } if name.name == "x"));
        assert!(matches!(*right, Expression::Variable { ref name, .. } if name.name == "y"));
    } else {
        panic!("Expected Add expression");
    }
}

#[test]
fn test_parse_binary_error_missing_close_brace() {
    let mut parser = parser_from_source("{1 + 2");
    let result = parser.parse_expression();

    assert!(result.is_err());
}

#[test]
fn test_parse_binary_error_missing_operator() {
    let mut parser = parser_from_source("{1 2}");
    let result = parser.parse_expression();

    assert!(result.is_err());
}

#[test]
fn test_parse_binary_error_missing_right_operand() {
    let mut parser = parser_from_source("{1 + }");
    let result = parser.parse_expression();

    assert!(result.is_err());
}

// ==================== Control Flow Tests ====================

#[test]
fn test_parse_return_with_value() {
    let mut parser = parser_from_source("return 42;");
    let result = parser.parse_return_statement();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Return { value, .. } = stmt {
        assert!(value.is_some());
        assert!(matches!(
            *value.unwrap(),
            Expression::IntegerLiteral { value: 42, .. }
        ));
    } else {
        panic!("Expected Return statement");
    }
}

#[test]
fn test_parse_return_without_value() {
    let mut parser = parser_from_source("return;");
    let result = parser.parse_return_statement();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Return { value, .. } = stmt {
        assert!(value.is_none());
    } else {
        panic!("Expected Return statement");
    }
}

#[test]
fn test_parse_return_with_expression() {
    let mut parser = parser_from_source("return {x + y};");
    let result = parser.parse_return_statement();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Return { value, .. } = stmt {
        assert!(value.is_some());
        assert!(matches!(*value.unwrap(), Expression::Add { .. }));
    } else {
        panic!("Expected Return statement");
    }
}

#[test]
fn test_parse_if_simple() {
    let mut parser = parser_from_source("if true { }");
    let result = parser.parse_if_statement();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::If {
        condition,
        else_ifs,
        else_block,
        ..
    } = stmt
    {
        assert!(matches!(
            *condition,
            Expression::BooleanLiteral { value: true, .. }
        ));
        assert!(else_ifs.is_empty());
        assert!(else_block.is_none());
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_parse_if_with_else() {
    let mut parser = parser_from_source("if x { } else { }");
    let result = parser.parse_if_statement();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::If { else_block, .. } = stmt {
        assert!(else_block.is_some());
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_parse_if_with_else_if() {
    let mut parser = parser_from_source("if x { } else if y { } else { }");
    let result = parser.parse_if_statement();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::If {
        else_ifs,
        else_block,
        ..
    } = stmt
    {
        assert_eq!(else_ifs.len(), 1);
        assert!(else_block.is_some());
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_parse_if_with_body() {
    let mut parser = parser_from_source("if {x > 0} { return x; }");
    let result = parser.parse_if_statement();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::If { then_block, .. } = stmt {
        assert_eq!(then_block.statements.len(), 1);
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_parse_while_loop() {
    let mut parser = parser_from_source("while true { }");
    let result = parser.parse_while_loop();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::WhileLoop { condition, .. } = stmt {
        assert!(matches!(
            *condition,
            Expression::BooleanLiteral { value: true, .. }
        ));
    } else {
        panic!("Expected WhileLoop statement");
    }
}

#[test]
fn test_parse_while_with_condition() {
    let mut parser = parser_from_source("while {i < 10} { }");
    let result = parser.parse_while_loop();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(matches!(stmt, Statement::WhileLoop { .. }));
}

#[test]
fn test_parse_while_with_body() {
    let mut parser = parser_from_source("while {x > 0} { x = {x - 1}; }");
    let result = parser.parse_while_loop();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::WhileLoop { body, .. } = stmt {
        assert_eq!(body.statements.len(), 1);
    } else {
        panic!("Expected WhileLoop statement");
    }
}

#[test]
fn test_parse_break() {
    let mut parser = parser_from_source("break;");
    let result = parser.parse_break_statement();

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Statement::Break { .. }));
}

#[test]
fn test_parse_continue() {
    let mut parser = parser_from_source("continue;");
    let result = parser.parse_continue_statement();

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Statement::Continue { .. }));
}

#[test]
fn test_parse_block_empty() {
    let mut parser = parser_from_source("{ }");
    let result = parser.parse_block();

    assert!(result.is_ok());
    let block = result.unwrap();
    assert!(block.statements.is_empty());
}

#[test]
fn test_parse_block_with_statements() {
    let mut parser = parser_from_source("{ let x: Int = 1; let y: Int = 2; }");
    let result = parser.parse_block();

    assert!(result.is_ok());
    let block = result.unwrap();
    assert_eq!(block.statements.len(), 2);
}

#[test]
fn test_parse_statement_let() {
    let mut parser = parser_from_source("let x: Int = 42;");
    let result = parser.parse_statement();

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Statement::VariableDeclaration { .. }
    ));
}

#[test]
fn test_parse_statement_return() {
    let mut parser = parser_from_source("return 0;");
    let result = parser.parse_statement();

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Statement::Return { .. }));
}

#[test]
fn test_parse_statement_if() {
    let mut parser = parser_from_source("if true { }");
    let result = parser.parse_statement();

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Statement::If { .. }));
}

#[test]
fn test_parse_statement_while() {
    let mut parser = parser_from_source("while true { }");
    let result = parser.parse_statement();

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Statement::WhileLoop { .. }));
}

#[test]
fn test_parse_statement_assignment() {
    let mut parser = parser_from_source("x = 42;");
    let result = parser.parse_statement();

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Statement::Assignment { .. }));
}

// ==================== Struct Parsing Tests ====================

#[test]
fn test_parse_struct_simple() {
    let mut parser = parser_from_source("struct Point { x: Float, y: Float }");
    let result = parser.parse_struct();

    assert!(result.is_ok());
    let typedef = result.unwrap();
    if let TypeDefinition::Structured { name, fields, .. } = typedef {
        assert_eq!(name.name, "Point");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name.name, "x");
        assert_eq!(fields[1].name.name, "y");
    } else {
        panic!("Expected Structured type");
    }
}

#[test]
fn test_parse_struct_empty() {
    let mut parser = parser_from_source("struct Empty { }");
    let result = parser.parse_struct();

    assert!(result.is_ok());
    let typedef = result.unwrap();
    if let TypeDefinition::Structured { name, fields, .. } = typedef {
        assert_eq!(name.name, "Empty");
        assert!(fields.is_empty());
    } else {
        panic!("Expected Structured type");
    }
}

#[test]
fn test_parse_struct_single_field() {
    let mut parser = parser_from_source("struct Wrapper { value: Int }");
    let result = parser.parse_struct();

    assert!(result.is_ok());
    let typedef = result.unwrap();
    if let TypeDefinition::Structured { fields, .. } = typedef {
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name.name, "value");
    } else {
        panic!("Expected Structured type");
    }
}

#[test]
fn test_parse_struct_complex_types() {
    let mut parser =
        parser_from_source("struct Data { items: Array<Int>, lookup: Map<String, Int> }");
    let result = parser.parse_struct();

    assert!(result.is_ok());
    let typedef = result.unwrap();
    if let TypeDefinition::Structured { fields, .. } = typedef {
        assert_eq!(fields.len(), 2);
        assert!(matches!(*fields[0].field_type, TypeSpecifier::Array { .. }));
        assert!(matches!(*fields[1].field_type, TypeSpecifier::Map { .. }));
    } else {
        panic!("Expected Structured type");
    }
}

#[test]
fn test_parse_struct_error_missing_brace() {
    let mut parser = parser_from_source("struct Point x: Float; }");
    let result = parser.parse_struct();

    assert!(result.is_err());
}

#[test]
fn test_parse_struct_error_missing_field_type() {
    let mut parser = parser_from_source("struct Point { x; }");
    let result = parser.parse_struct();

    assert!(result.is_err());
}

#[test]
fn test_parse_struct_single_field_no_trailing_comma() {
    // With comma-separated fields and optional trailing comma, this should succeed
    let mut parser = parser_from_source("struct Point { x: Float }");
    let result = parser.parse_struct();

    assert!(result.is_ok());
}

// ==================== Enum Parsing Tests ====================

#[test]
fn test_parse_enum_simple() {
    let mut parser = parser_from_source("enum Color { case Red; case Green; case Blue; }");
    let result = parser.parse_enum();

    assert!(result.is_ok());
    let typedef = result.unwrap();
    if let TypeDefinition::Enumeration { name, variants, .. } = typedef {
        assert_eq!(name.name, "Color");
        assert_eq!(variants.len(), 3);
        assert_eq!(variants[0].name.name, "Red");
        assert_eq!(variants[1].name.name, "Green");
        assert_eq!(variants[2].name.name, "Blue");
    } else {
        panic!("Expected Enumeration type");
    }
}

#[test]
fn test_parse_enum_empty() {
    let mut parser = parser_from_source("enum Empty { }");
    let result = parser.parse_enum();

    assert!(result.is_ok());
    let typedef = result.unwrap();
    if let TypeDefinition::Enumeration { variants, .. } = typedef {
        assert!(variants.is_empty());
    } else {
        panic!("Expected Enumeration type");
    }
}

#[test]
fn test_parse_enum_with_associated_type() {
    let mut parser = parser_from_source("enum Result { case Ok(Int); case Error(String); }");
    let result = parser.parse_enum();

    assert!(result.is_ok());
    let typedef = result.unwrap();
    if let TypeDefinition::Enumeration { name, variants, .. } = typedef {
        assert_eq!(name.name, "Result");
        assert_eq!(variants.len(), 2);
        assert!(!variants[0].associated_types.is_empty());
        assert!(!variants[1].associated_types.is_empty());
    } else {
        panic!("Expected Enumeration type");
    }
}

#[test]
fn test_parse_enum_mixed_variants() {
    let mut parser = parser_from_source("enum Option { case Some(Int); case None; }");
    let result = parser.parse_enum();

    assert!(result.is_ok());
    let typedef = result.unwrap();
    if let TypeDefinition::Enumeration { variants, .. } = typedef {
        assert_eq!(variants.len(), 2);
        assert!(!variants[0].associated_types.is_empty());
        assert!(variants[1].associated_types.is_empty());
    } else {
        panic!("Expected Enumeration type");
    }
}

#[test]
fn test_parse_enum_error_missing_case() {
    let mut parser = parser_from_source("enum Color { Red; }");
    let result = parser.parse_enum();

    assert!(result.is_err());
}

#[test]
fn test_parse_enum_error_missing_semicolon() {
    let mut parser = parser_from_source("enum Color { case Red }");
    let result = parser.parse_enum();

    assert!(result.is_err());
}

// ==================== Integration Tests ====================

#[test]
fn test_integration_function_with_body() {
    let source = r#"
func add(a: Int, b: Int) -> Int {
let result: Int = {a + b};
return result;
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.name.name, "add");
    assert_eq!(func.parameters.len(), 2);
    assert_eq!(func.body.statements.len(), 2);
}

#[test]
fn test_integration_function_with_control_flow() {
    let source = r#"
func abs(n: Int) -> Int {
if {n < 0} {
    return {0 - n};
} else {
    return n;
}
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.name.name, "abs");
    assert_eq!(func.body.statements.len(), 1);
    assert!(matches!(func.body.statements[0], Statement::If { .. }));
}

#[test]
fn test_integration_function_with_loop() {
    let source = r#"
func sum(n: Int) -> Int {
let mut total: Int = 0;
let mut i: Int = 0;
while {i < n} {
    total = {total + i};
    i = {i + 1};
}
return total;
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.body.statements.len(), 4); // 2 lets + while + return
}

#[test]
fn test_integration_struct_and_function() {
    let struct_source = "struct Point { x: Float, y: Float }";
    let mut parser = parser_from_source(struct_source);
    let struct_result = parser.parse_struct();
    assert!(struct_result.is_ok());

    let func_source = r#"
func distance(p: Point) -> Float {
return {p.x + p.y};
}
"#;
    let mut parser = parser_from_source(func_source);
    let func_result = parser.parse_function();
    assert!(func_result.is_ok(), "Parse error: {:?}", func_result.err());
}

#[test]
fn test_integration_nested_control_flow() {
    let source = r#"
func classify(n: Int) -> Int {
if {n > 0} {
    if {n > 100} {
        return 2;
    } else {
        return 1;
    }
} else if {n < 0} {
    return 0;
} else {
    return 0;
}
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    if let Statement::If {
        else_ifs,
        else_block,
        ..
    } = &func.body.statements[0]
    {
        assert_eq!(else_ifs.len(), 1);
        assert!(else_block.is_some());
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_integration_module_with_import() {
    let source = r#"
module math;

import std.io;
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_module();

    assert!(result.is_ok(), "Parse error: {:?}", result.err());
    let module = result.unwrap();
    assert_eq!(module.name.name, "math");
    assert_eq!(module.imports.len(), 1);
}

#[test]
fn test_integration_complex_expressions() {
    let source = r#"
func compute(a: Int, b: Int, c: Int) -> Int {
let x: Int = {a + b};
let y: Int = {x * c};
let z: Int = {{a + b} * {c - 1}};
return z;
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok(), "Parse error: {:?}", result.err());
    let func = result.unwrap();
    assert_eq!(func.body.statements.len(), 4);
}

#[test]
fn test_integration_array_operations() {
    let source = r#"
func process(items: Array<Int>) -> Int {
let first: Int = items[0];
items[0] = 42;
return first;
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok(), "Parse error: {:?}", result.err());
    let func = result.unwrap();
    assert_eq!(func.body.statements.len(), 3);
}

#[test]
fn test_integration_struct_field_access() {
    let source = r#"
func swap(p: Point) -> Point {
let temp: Float = p.x;
p.x = p.y;
p.y = temp;
return p;
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert_eq!(func.body.statements.len(), 4);
}

#[test]
fn test_integration_ownership_types() {
    let source = r#"
func take_owned(data: ^Data) -> Int {
return 0;
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert!(matches!(
        *func.parameters[0].param_type,
        TypeSpecifier::Owned { .. }
    ));
}

#[test]
fn test_integration_borrowed_types() {
    let source = r#"
func read_only(data: &Data) -> Int {
return 0;
}
"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_function();

    assert!(result.is_ok());
    let func = result.unwrap();
    assert!(matches!(
        *func.parameters[0].param_type,
        TypeSpecifier::Owned {
            ownership: OwnershipKind::Borrowed,
            ..
        }
    ));
}

#[test]
fn test_integration_external_function() {
    let source = r#"@extern(library: "libc") func puts(s: Pointer<Void>) -> Int;"#;
    let mut parser = parser_from_source(source);

    let annotation = parser.parse_annotation().unwrap();
    let result = parser.parse_external_function(annotation);

    assert!(result.is_ok());
    let ext_func = result.unwrap();
    assert_eq!(ext_func.name.name, "puts");
    assert_eq!(ext_func.library, "libc");
}

// ==================== ERROR RECOVERY TESTS ====================

#[test]
fn test_error_recovery_unexpected_token() {
    // Simple error: unexpected token at module level
    let source = r#"
module test;

123;

func good() -> Int {
return 0;
}
"#;
    let mut parser = parser_from_source(source);
    let (result, errors) = parser.parse_module_with_recovery();

    // Should have parsed something
    assert!(result.is_some());
    let module = result.unwrap();

    // Should have at least one error
    assert!(!errors.is_empty(), "Should have collected errors");

    // The good function should still be parsed
    assert!(module.function_definitions.len() >= 1);
    assert!(module
        .function_definitions
        .iter()
        .any(|f| f.name.name == "good"));
}

#[test]
fn test_error_recovery_with_valid_first_function() {
    // First function valid, second has extra tokens
    let source = r#"
module test;

func first() -> Int {
return 1;
}

struct;

func third() -> Int {
return 3;
}
"#;
    let mut parser = parser_from_source(source);
    let (result, errors) = parser.parse_module_with_recovery();

    assert!(result.is_some());
    // Should have errors from the incomplete struct
    assert!(!errors.is_empty());

    let module = result.unwrap();
    let func_names: Vec<_> = module
        .function_definitions
        .iter()
        .map(|f| f.name.name.as_str())
        .collect();

    // First function should definitely be parsed
    assert!(func_names.contains(&"first"), "Should have parsed 'first'");
}

#[test]
fn test_synchronize_simple() {
    // Test synchronization with simple invalid token
    let source = r#"
module test;

999;

func valid() -> Int {
return 42;
}
"#;
    let mut parser = parser_from_source(source);
    let (result, errors) = parser.parse_module_with_recovery();

    assert!(result.is_some());
    assert!(!errors.is_empty());

    // The valid function should be parsed after recovery
    let module = result.unwrap();
    assert!(module
        .function_definitions
        .iter()
        .any(|f| f.name.name == "valid"));
}

#[test]
fn test_error_recovery_inline_module() {
    // Test recovery in inline module syntax
    let source = r#"
module test {
func good() -> Int {
    return 1;
}

123;

func also_good() -> Int {
    return 2;
}
}
"#;
    let mut parser = parser_from_source(source);
    let (result, errors) = parser.parse_module_with_recovery();

    assert!(result.is_some());
    // Should have errors from the invalid token
    assert!(!errors.is_empty());

    let module = result.unwrap();
    // Should have parsed the good functions
    assert!(module.function_definitions.len() >= 1);
}

#[test]
fn test_synchronization_methods() {
    // Test that synchronization skips to the next declaration
    let source = r#"
module test;

456;

func next_item() -> Int {
return 0;
}
"#;
    let mut parser = parser_from_source(source);

    // Skip the module declaration
    parser.expect_keyword(Keyword::Module, "").unwrap();
    parser.parse_identifier().unwrap();
    parser.expect(&TokenType::Semicolon, "").unwrap();

    // Synchronize should skip the invalid token and stop at 'func'
    parser.synchronize_to_module_item();

    // Should now be at 'func' keyword
    assert!(parser.check_keyword(Keyword::Func));
}

#[test]
fn test_has_errors_and_take_errors() {
    let source = "module test;";
    let mut parser = parser_from_source(source);

    assert!(!parser.has_errors());

    parser.add_error(ParserError::UnexpectedEof {
        expected: "test".to_string(),
    });

    assert!(parser.has_errors());
    assert_eq!(parser.errors().len(), 1);

    let taken = parser.take_errors();
    assert_eq!(taken.len(), 1);
    assert!(!parser.has_errors());
}

// ==================== MATCH EXPRESSION TESTS ====================

#[test]
fn test_match_simple() {
    let source = r#"match x { 1 => 10, 2 => 20, _ => 0 }"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_match_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Match { cases, .. } = expr {
        assert_eq!(cases.len(), 3);
    } else {
        panic!("Expected Match expression");
    }
}

#[test]
fn test_match_with_variable_binding() {
    let source = r#"match value { x => x }"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_match_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Match { cases, .. } = expr {
        assert_eq!(cases.len(), 1);
        // x is a variable binding pattern
        assert!(matches!(cases[0].pattern, Pattern::Wildcard { .. }));
    } else {
        panic!("Expected Match expression");
    }
}

#[test]
fn test_match_with_enum_variant() {
    let source = r#"match opt { Some(x) => x, None => 0 }"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_match_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Match { cases, .. } = expr {
        assert_eq!(cases.len(), 2);
        // First case: Some(x)
        if let Pattern::EnumVariant {
            variant_name,
            bindings,
            ..
        } = &cases[0].pattern
        {
            assert_eq!(variant_name.name, "Some");
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].name, "x");
        } else {
            panic!("Expected EnumVariant pattern");
        }
    } else {
        panic!("Expected Match expression");
    }
}

#[test]
fn test_match_with_literal_patterns() {
    let source = r#"match s { "hello" => 1, "world" => 2, _ => 0 }"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_match_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Match { cases, .. } = expr {
        assert_eq!(cases.len(), 3);
        // First two should be literal patterns
        assert!(matches!(cases[0].pattern, Pattern::Literal { .. }));
        assert!(matches!(cases[1].pattern, Pattern::Literal { .. }));
        // Last should be wildcard
        assert!(matches!(
            cases[2].pattern,
            Pattern::Wildcard { binding: None, .. }
        ));
    } else {
        panic!("Expected Match expression");
    }
}

#[test]
fn test_match_with_bool_patterns() {
    let source = r#"match flag { true => 1, false => 0 }"#;
    let mut parser = parser_from_source(source);
    let result = parser.parse_match_expression();

    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Match { cases, .. } = expr {
        assert_eq!(cases.len(), 2);
        assert!(matches!(cases[0].pattern, Pattern::Literal { .. }));
        assert!(matches!(cases[1].pattern, Pattern::Literal { .. }));
    } else {
        panic!("Expected Match expression");
    }
}

#[test]
fn test_fat_arrow_token() {
    let source = "=>";
    let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
    let tokens = lexer.tokenize().unwrap();
    assert!(matches!(tokens[0].token_type, TokenType::FatArrow));
}

#[test]
fn test_underscore_token() {
    let source = "_";
    let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
    let tokens = lexer.tokenize().unwrap();
    assert!(matches!(tokens[0].token_type, TokenType::Underscore));
}

#[test]
fn test_underscore_in_identifier() {
    let source = "_foo";
    let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
    let tokens = lexer.tokenize().unwrap();
    // _foo should be an identifier, not underscore
    assert!(matches!(tokens[0].token_type, TokenType::Identifier(_)));
}

// ==================== For Loop Tests ====================

#[test]
fn test_for_loop_simple() {
    let source = "for x in items { }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_for_loop();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::ForEachLoop {
        element_binding,
        collection,
        ..
    } = stmt
    {
        assert_eq!(element_binding.name, "x");
        assert!(matches!(*collection, Expression::Variable { .. }));
    } else {
        panic!("Expected ForEachLoop statement");
    }
}

#[test]
fn test_for_loop_with_type_annotation() {
    let source = "for x: Int in numbers { }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_for_loop();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::ForEachLoop {
        element_binding,
        element_type,
        ..
    } = stmt
    {
        assert_eq!(element_binding.name, "x");
        assert!(matches!(
            *element_type,
            TypeSpecifier::Primitive { .. } | TypeSpecifier::Named { .. }
        ));
    } else {
        panic!("Expected ForEachLoop statement");
    }
}

#[test]
fn test_for_loop_with_body() {
    let source = "for item in list { let x = item; }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_for_loop();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::ForEachLoop { body, .. } = stmt {
        assert_eq!(body.statements.len(), 1);
    } else {
        panic!("Expected ForEachLoop statement");
    }
}

#[test]
fn test_for_loop_in_statement() {
    let source = "for i in items { break; }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_statement();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let stmt = result.unwrap();
    assert!(matches!(stmt, Statement::ForEachLoop { .. }));
}

#[test]
fn test_for_loop_with_function_call_collection() {
    let source = "for x in get_items() { }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_for_loop();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::ForEachLoop { collection, .. } = stmt {
        assert!(matches!(*collection, Expression::FunctionCall { .. }));
    } else {
        panic!("Expected ForEachLoop statement");
    }
}

#[test]
fn test_for_loop_nested() {
    let source = "for i in outer { for j in inner { } }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_for_loop();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::ForEachLoop { body, .. } = stmt {
        assert_eq!(body.statements.len(), 1);
        assert!(matches!(body.statements[0], Statement::ForEachLoop { .. }));
    } else {
        panic!("Expected ForEachLoop statement");
    }
}

// ==================== Lambda Tests ====================

#[test]
fn test_lambda_zero_params() {
    let source = "() => 42";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda {
        parameters, body, ..
    } = expr
    {
        assert_eq!(parameters.len(), 0);
        assert!(matches!(body, LambdaBody::Expression(_)));
    } else {
        panic!("Expected Lambda expression, got {:?}", expr);
    }
}

#[test]
fn test_lambda_single_param_typed() {
    let source = "(x: Int) => x";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda {
        parameters, body, ..
    } = expr
    {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name.name, "x");
        assert!(matches!(body, LambdaBody::Expression(_)));
    } else {
        panic!("Expected Lambda expression, got {:?}", expr);
    }
}

#[test]
fn test_lambda_multiple_params() {
    // Expression body with a simple identifier
    let source = "(x: Int, y: Int) => x";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda {
        parameters, body, ..
    } = expr
    {
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name.name, "x");
        assert_eq!(parameters[1].name.name, "y");
        assert!(matches!(body, LambdaBody::Expression(_)));
    } else {
        panic!("Expected Lambda expression, got {:?}", expr);
    }
}

#[test]
fn test_lambda_with_block_body() {
    let source = "(x: Int) => { let y = x; return y; }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda { body, .. } = expr {
        assert!(matches!(body, LambdaBody::Block(_)));
        if let LambdaBody::Block(block) = body {
            assert_eq!(block.statements.len(), 2);
        }
    } else {
        panic!("Expected Lambda expression, got {:?}", expr);
    }
}

#[test]
fn test_lambda_with_return_type() {
    let source = "(x: Int) -> Int => x";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda { return_type, .. } = expr {
        assert!(return_type.is_some());
    } else {
        panic!("Expected Lambda expression, got {:?}", expr);
    }
}

#[test]
fn test_parenthesized_expression() {
    // (42) should be parsed as a parenthesized expression, not a lambda
    let source = "(42)";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    assert!(
        matches!(expr, Expression::IntegerLiteral { value: 42, .. }),
        "Expected IntegerLiteral, got {:?}",
        expr
    );
}

#[test]
fn test_parenthesized_binary_expression() {
    let source = "({a + b})";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    // The inner expression should be an Add
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Add { .. }));
}

// ==================== Method Call Tests ====================

#[test]
fn test_method_call_no_args() {
    let source = "obj.method()";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::MethodCall {
        receiver,
        method_name,
        arguments,
        ..
    } = expr
    {
        assert!(matches!(*receiver, Expression::Variable { .. }));
        assert_eq!(method_name.name, "method");
        assert_eq!(arguments.len(), 0);
    } else {
        panic!("Expected MethodCall expression, got {:?}", expr);
    }
}

#[test]
fn test_method_call_with_args() {
    let source = "list.push(42)";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::MethodCall {
        receiver,
        method_name,
        arguments,
        ..
    } = expr
    {
        assert!(matches!(*receiver, Expression::Variable { .. }));
        assert_eq!(method_name.name, "push");
        assert_eq!(arguments.len(), 1);
    } else {
        panic!("Expected MethodCall expression, got {:?}", expr);
    }
}

#[test]
fn test_method_call_multiple_args() {
    let source = "map.insert(key, value)";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::MethodCall {
        method_name,
        arguments,
        ..
    } = expr
    {
        assert_eq!(method_name.name, "insert");
        assert_eq!(arguments.len(), 2);
    } else {
        panic!("Expected MethodCall expression, got {:?}", expr);
    }
}

#[test]
fn test_method_call_chained() {
    let source = "obj.first().second()";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    // Outer call should be second()
    if let Expression::MethodCall {
        receiver,
        method_name,
        ..
    } = expr
    {
        assert_eq!(method_name.name, "second");
        // Inner call should be first()
        if let Expression::MethodCall {
            method_name: inner_method,
            ..
        } = *receiver
        {
            assert_eq!(inner_method.name, "first");
        } else {
            panic!("Expected inner MethodCall");
        }
    } else {
        panic!("Expected MethodCall expression, got {:?}", expr);
    }
}

#[test]
fn test_field_access_still_works() {
    let source = "obj.field";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    assert!(
        matches!(expr, Expression::FieldAccess { .. }),
        "Expected FieldAccess, got {:?}",
        expr
    );
}

#[test]
fn test_method_call_on_field() {
    let source = "obj.field.method()";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::MethodCall {
        receiver,
        method_name,
        ..
    } = expr
    {
        assert_eq!(method_name.name, "method");
        // Receiver should be a field access
        assert!(
            matches!(*receiver, Expression::FieldAccess { .. }),
            "Expected FieldAccess receiver"
        );
    } else {
        panic!("Expected MethodCall expression, got {:?}", expr);
    }
}

// ==================== Range Expression Tests ====================

#[test]
fn test_range_exclusive() {
    let source = "0..10";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Range {
        start,
        end,
        inclusive,
        ..
    } = expr
    {
        assert!(start.is_some());
        assert!(end.is_some());
        assert!(!inclusive);
    } else {
        panic!("Expected Range expression, got {:?}", expr);
    }
}

#[test]
fn test_range_inclusive() {
    let source = "0..=10";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Range {
        start,
        end,
        inclusive,
        ..
    } = expr
    {
        assert!(start.is_some());
        assert!(end.is_some());
        assert!(inclusive);
    } else {
        panic!("Expected Range expression, got {:?}", expr);
    }
}

#[test]
fn test_range_prefix() {
    let source = "..10";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Range { start, end, .. } = expr {
        assert!(start.is_none());
        assert!(end.is_some());
    } else {
        panic!("Expected Range expression, got {:?}", expr);
    }
}

#[test]
fn test_range_postfix() {
    let source = "0..";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();

    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Range { start, end, .. } = expr {
        assert!(start.is_some());
        assert!(end.is_none());
    } else {
        panic!("Expected Range expression, got {:?}", expr);
    }
}

#[test]
fn test_dotdot_token() {
    let source = "..";
    let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
    let tokens = lexer.tokenize().unwrap();
    assert!(matches!(tokens[0].token_type, TokenType::DotDot));
}

#[test]
fn test_dotdotequal_token() {
    let source = "..=";
    let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
    let tokens = lexer.tokenize().unwrap();
    assert!(matches!(tokens[0].token_type, TokenType::DotDotEqual));
}

// ========== Edge Case Tests ==========

// Range edge cases
#[test]
fn test_range_prefix_in_expression() {
    // Prefix range: ..end directly parsed
    let source = "..10";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Range {
        start,
        end,
        inclusive,
        ..
    } = expr
    {
        assert!(start.is_none());
        assert!(end.is_some());
        assert!(!inclusive);
    } else {
        panic!("Expected Range expression");
    }
}

#[test]
fn test_range_prefix_inclusive() {
    // Prefix inclusive range: ..=end
    let source = "..=5";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Range {
        start, inclusive, ..
    } = expr
    {
        assert!(start.is_none());
        assert!(inclusive);
    } else {
        panic!("Expected Range expression");
    }
}

#[test]
fn test_for_loop_with_range() {
    // For loop iterating over a range - now produces FixedIterationLoop
    let source = "for i in 0..10 { x = i; }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_for_loop();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::FixedIterationLoop { counter, inclusive, .. } = stmt {
        assert_eq!(counter.name, "i");
        assert!(!inclusive); // exclusive range
    } else {
        panic!("Expected FixedIterationLoop statement");
    }
}

#[test]
fn test_for_loop_with_inclusive_range() {
    // For loop with inclusive range - now produces FixedIterationLoop
    let source = "for i in 1..=5 { x = i; }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_for_loop();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::FixedIterationLoop { counter, inclusive, .. } = stmt {
        assert_eq!(counter.name, "i");
        assert!(inclusive); // inclusive range
    } else {
        panic!("Expected FixedIterationLoop statement");
    }
}

// Lambda edge cases
#[test]
fn test_lambda_typed_param_direct() {
    // Lambda with typed parameter, parsed directly
    let source = "(x: Int) => x";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Lambda { parameters, .. } = expr {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name.name, "x");
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_lambda_untyped_param_direct() {
    // Lambda with untyped parameter, parsed directly
    let source = "(x) => x";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Lambda { parameters, .. } = expr {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name.name, "x");
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_lambda_with_block_body_direct() {
    // Lambda with block body, parsed directly
    let source = "(x: Int) => { return {x + 1}; }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Lambda { body, .. } = expr {
        assert!(matches!(body, LambdaBody::Block(_)));
    } else {
        panic!("Expected Lambda expression");
    }
}

// Method call edge cases
#[test]
fn test_method_call_in_braced_expression() {
    // Method call inside a braced binary expression
    let source = "{a.len() + b.len()}";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Add { left, right, .. } = expr {
        assert!(matches!(*left, Expression::MethodCall { .. }));
        assert!(matches!(*right, Expression::MethodCall { .. }));
    } else {
        panic!("Expected Add expression");
    }
}

#[test]
fn test_method_call_deeply_chained() {
    // Deeply chained method calls - parsed directly
    let source = "a.first().second().third().fourth()";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    // Should be MethodCall for fourth()
    if let Expression::MethodCall {
        method_name,
        receiver,
        ..
    } = expr
    {
        assert_eq!(method_name.name, "fourth");
        // Receiver should be MethodCall for third()
        assert!(matches!(*receiver, Expression::MethodCall { .. }));
    } else {
        panic!("Expected MethodCall expression");
    }
}

#[test]
fn test_method_call_with_expression_arg() {
    // Method call with braced expression as argument
    let source = "list.get({i + 1})";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::MethodCall {
        method_name,
        arguments,
        ..
    } = expr
    {
        assert_eq!(method_name.name, "get");
        assert_eq!(arguments.len(), 1);
    } else {
        panic!("Expected MethodCall expression");
    }
}

// Match expression edge cases
#[test]
fn test_match_with_bool_result() {
    // Match expression with bool patterns - parsed directly
    let source = "match x { 1 => true, _ => false }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_match_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Match { cases, .. } = expr {
        assert_eq!(cases.len(), 2);
    } else {
        panic!("Expected Match expression");
    }
}

#[test]
fn test_match_with_multiple_enum_variants() {
    // Match with multiple enum variant patterns - parsed directly
    let source = "match opt { Some(x) => x, None => 0 }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_match_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Match { cases, .. } = expr {
        assert_eq!(cases.len(), 2);
        // First case: Some(x)
        assert!(matches!(cases[0].pattern, Pattern::EnumVariant { .. }));
    } else {
        panic!("Expected Match expression");
    }
}

// Combined feature tests
#[test]
fn test_combined_for_range() {
    // For loop with integer range - produces FixedIterationLoop
    let source = "for i in 0..10 { x = i; }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_for_loop();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::FixedIterationLoop { counter, inclusive, .. } = stmt {
        assert_eq!(counter.name, "i");
        assert!(!inclusive);
    } else {
        panic!("Expected FixedIterationLoop statement");
    }
}

#[test]
fn test_combined_method_with_lambda() {
    // Method call with lambda as argument - parsed directly
    // Note: uses braced expression for lambda body to work with parser
    let source = "list.map((x) => x)";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::MethodCall {
        method_name,
        arguments,
        ..
    } = expr
    {
        assert_eq!(method_name.name, "map");
        assert_eq!(arguments.len(), 1);
        // The argument should be a lambda
        assert!(matches!(&*arguments[0].value, Expression::Lambda { .. }));
    } else {
        panic!("Expected MethodCall expression");
    }
}

#[test]
fn test_combined_match_multiple_cases() {
    // Match with wildcard and enum patterns
    let source = "match status { Ok(v) => v, Err(e) => 0, _ => default }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_match_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Match { cases, .. } = expr {
        assert_eq!(cases.len(), 3);
    } else {
        panic!("Expected Match expression");
    }
}

#[test]
fn test_combined_nested_for_range() {
    // For loop with integer range - verify statement structure
    let source = "for i in 1..=100 { count = {count + 1}; }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_for_loop();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::FixedIterationLoop {
        counter,
        body,
        inclusive,
        ..
    } = stmt
    {
        assert_eq!(counter.name, "i");
        assert!(inclusive); // inclusive range
        assert!(!body.statements.is_empty());
    } else {
        panic!("Expected FixedIterationLoop statement");
    }
}

#[test]
fn test_return_lambda_direct() {
    // Return statement with lambda value - parsed via return parsing
    let source = "return (x) => x;";
    let mut parser = parser_from_source(source);
    let result = parser.parse_return_statement();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Return {
        value: Some(expr), ..
    } = stmt
    {
        assert!(matches!(*expr, Expression::Lambda { .. }));
    } else {
        panic!("Expected Return statement with Lambda");
    }
}

#[test]
fn test_method_call_result() {
    // Method call on expression
    let source = "callbacks.get(0)";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::MethodCall { .. }));
}

#[test]
fn test_complex_expression_chain() {
    // Complex chain: field access + method call + array access
    let source = "obj.field.method()[0]";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    // Outermost should be array access
    assert!(matches!(expr, Expression::ArrayAccess { .. }));
}

// ========== Closure Capture Tests ==========

#[test]
fn test_closure_single_capture_by_value() {
    // Closure with single capture by value
    let source = "[x](y) => y";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda {
        captures,
        parameters,
        ..
    } = expr
    {
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].name.name, "x");
        assert!(matches!(captures[0].mode, CaptureMode::ByValue));
        assert_eq!(parameters.len(), 1);
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_closure_single_capture_by_reference() {
    // Closure with single capture by reference
    let source = "[&x](y) => y";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda { captures, .. } = expr {
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].name.name, "x");
        assert!(matches!(captures[0].mode, CaptureMode::ByReference));
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_closure_capture_by_mut_reference() {
    // Closure with capture by mutable reference
    let source = "[&mut counter]() => counter";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda {
        captures,
        parameters,
        ..
    } = expr
    {
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].name.name, "counter");
        assert!(matches!(captures[0].mode, CaptureMode::ByMutableReference));
        assert_eq!(parameters.len(), 0);
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_closure_multiple_captures() {
    // Closure with multiple captures of different modes
    let source = "[x, &y, &mut z](a: Int) => a";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda {
        captures,
        parameters,
        ..
    } = expr
    {
        assert_eq!(captures.len(), 3);
        assert_eq!(captures[0].name.name, "x");
        assert!(matches!(captures[0].mode, CaptureMode::ByValue));
        assert_eq!(captures[1].name.name, "y");
        assert!(matches!(captures[1].mode, CaptureMode::ByReference));
        assert_eq!(captures[2].name.name, "z");
        assert!(matches!(captures[2].mode, CaptureMode::ByMutableReference));
        assert_eq!(parameters.len(), 1);
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_closure_empty_captures() {
    // Closure with empty capture list (explicit no captures)
    let source = "[](x) => x";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda {
        captures,
        parameters,
        ..
    } = expr
    {
        assert_eq!(captures.len(), 0);
        assert_eq!(parameters.len(), 1);
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_closure_with_block_body() {
    // Closure with captures and block body
    let source = "[total](x: Int) => { return {total + x}; }";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda { captures, body, .. } = expr {
        assert_eq!(captures.len(), 1);
        assert!(matches!(body, LambdaBody::Block(_)));
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_closure_with_return_type() {
    // Closure with captures and explicit return type
    let source = "[state](x: Int) -> Int => x";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Error: {:?}", result.err());
    let expr = result.unwrap();
    if let Expression::Lambda {
        captures,
        return_type,
        ..
    } = expr
    {
        assert_eq!(captures.len(), 1);
        assert!(return_type.is_some());
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_lambda_without_captures_still_works() {
    // Verify lambdas without captures still work (backward compatibility)
    let source = "(x: Int) => x";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::Lambda { captures, .. } = expr {
        assert_eq!(captures.len(), 0);
    } else {
        panic!("Expected Lambda expression");
    }
}

#[test]
fn test_function_call_with_labeled_arguments() {
    let source = "myFunc(label: value, other: 123)";
    let mut parser = parser_from_source(source);
    let result = parser.parse_expression();
    assert!(result.is_ok());
    let expr = result.unwrap();
    if let Expression::FunctionCall { call, .. } = expr {
        assert_eq!(call.arguments.len(), 2);
        assert_eq!(call.arguments[0].parameter_name.name, "label");
        assert_eq!(call.arguments[1].parameter_name.name, "other");
    } else {
        panic!("Expected FunctionCall expression");
    }
}

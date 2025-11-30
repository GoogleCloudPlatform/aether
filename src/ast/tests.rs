use super::*;

#[test]
fn test_identifier_creation() {
    let loc = SourceLocation::new("test.aether".to_string(), 1, 1, 0);
    let id = Identifier::new("test_identifier".to_string(), loc.clone());

    assert_eq!(id.name, "test_identifier");
    assert_eq!(id.source_location, loc);
}

#[test]
fn test_ast_pretty_printer() {
    let mut printer = ASTPrettyPrinter::new();
    let loc = SourceLocation::new("test.aether".to_string(), 1, 1, 0);

    let module = Module {
        name: Identifier::new("test_module".to_string(), loc.clone()),
        intent: Some("Test module".to_string()),
        imports: vec![],
        exports: vec![],
        type_definitions: vec![],
        constant_declarations: vec![],
        function_definitions: vec![],
        external_functions: vec![],
        source_location: loc,
    };

    let output = printer.print_module(&module);
    assert!(output.contains("Module 'test_module'"));
    assert!(output.contains("intent: \"Test module\""));
}

#[test]
fn test_expression_serialization() {
    let loc = SourceLocation::new("test.aether".to_string(), 1, 1, 0);
    let expr = Expression::IntegerLiteral {
        value: 42,
        source_location: loc,
    };

    let serialized = serde_json::to_string(&expr).unwrap();
    let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        Expression::IntegerLiteral { value, .. } => assert_eq!(value, 42),
        _ => panic!("Deserialization failed"),
    }
}

use super::*;
use crate::ast::PrimitiveType;
use crate::ast::{self, Identifier};

#[test]
fn test_simple_function_lowering() {
    let mut ctx = LoweringContext::new();

    // Create a simple AST function
    let ast_func = ast::Function {
        name: ast::Identifier::new("test".to_string(), crate::error::SourceLocation::unknown()),
        intent: None,
        generic_parameters: vec![],
        lifetime_parameters: vec![],
        parameters: vec![],
        return_type: Box::new(ast::TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: SourceLocation::unknown(),
        }),
        metadata: ast::FunctionMetadata {
            preconditions: vec![],
            postconditions: vec![],
            invariants: vec![],
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: vec![],
            thread_safe: None,
            may_block: None,
        },
        body: ast::Block {
            statements: vec![ast::Statement::Return {
                value: Some(Box::new(ast::Expression::IntegerLiteral {
                    value: 42,
                    source_location: SourceLocation::unknown(),
                })),
                source_location: SourceLocation::unknown(),
            }],
            source_location: SourceLocation::unknown(),
        },
        export_info: None,
        is_async: false,
        source_location: crate::error::SourceLocation::unknown(),
    };

    ctx.lower_function(&ast_func)
        .expect("Lowering should succeed");

    assert!(ctx.program.functions.contains_key("test"));
    let mir_func = &ctx.program.functions["test"];
    assert_eq!(mir_func.name, "test");
    assert_eq!(mir_func.basic_blocks.len(), 1);
}

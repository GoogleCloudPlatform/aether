//! Integration tests for optimization passes

use aether::optimizations::{OptimizationManager, OptimizationPass};
use aether::optimizations::constant_folding::ConstantFoldingPass;
use aether::optimizations::dead_code_elimination::DeadCodeEliminationPass;
use aether::optimizations::common_subexpression::CommonSubexpressionEliminationPass;
use aether::mir::{
    Builder, Function, Program, Statement, Rvalue, Operand, Constant, ConstantValue,
    Place, SourceInfo, BinOp, Terminator,
};
use aether::types::Type;
use aether::ast::PrimitiveType;
use aether::error::SourceLocation;
use std::collections::HashMap;

/// Create a test function with constant operations
fn create_constant_folding_test_function() -> Function {
    let mut builder = Builder::new();
    
    builder.start_function(
        "constant_test".to_string(),
        vec![],
        Type::primitive(PrimitiveType::Integer),
    );
    
    let temp1 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    let temp2 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    let temp3 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    
    // temp1 = 2 + 3
    builder.push_statement(Statement::Assign {
        place: Place { local: temp1, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Add,
            left: Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(2),
            }),
            right: Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(3),
            }),
        },
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    
    // temp2 = 4 * 5
    builder.push_statement(Statement::Assign {
        place: Place { local: temp2, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Mul,
            left: Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(4),
            }),
            right: Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(5),
            }),
        },
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    
    // temp3 = temp1 + temp2 (uses previous results)
    builder.push_statement(Statement::Assign {
        place: Place { local: temp3, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Add,
            left: Operand::Copy(Place { local: temp1, projection: vec![] }),
            right: Operand::Copy(Place { local: temp2, projection: vec![] }),
        },
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    
    builder.set_terminator(Terminator::Return);
    builder.finish_function()
}

/// Create a test function with dead code
fn create_dead_code_test_function() -> Function {
    let mut builder = Builder::new();
    
    builder.start_function(
        "dead_code_test".to_string(),
        vec![],
        Type::primitive(PrimitiveType::Integer),
    );
    
    let dead_local = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    let live_local = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    
    // Dead assignment
    builder.push_statement(Statement::Assign {
        place: Place { local: dead_local, projection: vec![] },
        rvalue: Rvalue::Use(Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(42),
        })),
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    
    // Live assignment (though still not actually used in return)
    builder.push_statement(Statement::Assign {
        place: Place { local: live_local, projection: vec![] },
        rvalue: Rvalue::Use(Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(24),
        })),
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    
    // Create unreachable block
    let unreachable_block = builder.new_block();
    builder.switch_to_block(unreachable_block);
    builder.push_statement(Statement::Assign {
        place: Place { local: dead_local, projection: vec![] },
        rvalue: Rvalue::Use(Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(999),
        })),
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    builder.set_terminator(Terminator::Return);
    
    // Switch back to main block and return
    builder.switch_to_block(0);
    builder.set_terminator(Terminator::Return);
    
    builder.finish_function()
}

/// Create a test function with common subexpressions
fn create_cse_test_function() -> Function {
    let mut builder = Builder::new();
    
    builder.start_function(
        "cse_test".to_string(),
        vec![],
        Type::primitive(PrimitiveType::Integer),
    );
    
    let a = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    let b = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    let temp1 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    let temp2 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    
    // a = 10
    builder.push_statement(Statement::Assign {
        place: Place { local: a, projection: vec![] },
        rvalue: Rvalue::Use(Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(10),
        })),
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    
    // b = 20
    builder.push_statement(Statement::Assign {
        place: Place { local: b, projection: vec![] },
        rvalue: Rvalue::Use(Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(20),
        })),
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    
    // temp1 = a + b
    builder.push_statement(Statement::Assign {
        place: Place { local: temp1, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Add,
            left: Operand::Copy(Place { local: a, projection: vec![] }),
            right: Operand::Copy(Place { local: b, projection: vec![] }),
        },
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    
    // temp2 = a + b (common subexpression)
    builder.push_statement(Statement::Assign {
        place: Place { local: temp2, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Add,
            left: Operand::Copy(Place { local: a, projection: vec![] }),
            right: Operand::Copy(Place { local: b, projection: vec![] }),
        },
        source_info: SourceInfo {
            span: SourceLocation::unknown(),
            scope: 0,
        },
    });
    
    builder.set_terminator(Terminator::Return);
    builder.finish_function()
}

#[test]
fn test_constant_folding_optimization() {
    let mut function = create_constant_folding_test_function();
    let mut pass = ConstantFoldingPass::new();
    
    // Count constant expressions before optimization
    let original_binary_ops = count_binary_operations(&function);
    
    // Run constant folding
    let changed = pass.run_on_function(&mut function).unwrap();
    assert!(changed);
    
    // Count constant expressions after optimization
    let optimized_binary_ops = count_binary_operations(&function);
    
    // Should have fewer binary operations due to constant folding
    assert!(optimized_binary_ops < original_binary_ops);
}

#[test]
fn test_dead_code_elimination_optimization() {
    let mut function = create_dead_code_test_function();
    let mut pass = DeadCodeEliminationPass::new();
    
    let original_block_count = function.basic_blocks.len();
    let original_statement_count: usize = function.basic_blocks.values()
        .map(|block| block.statements.len())
        .sum();
    
    // Run dead code elimination
    let changed = pass.run_on_function(&mut function).unwrap();
    assert!(changed);
    
    let optimized_block_count = function.basic_blocks.len();
    let optimized_statement_count: usize = function.basic_blocks.values()
        .map(|block| block.statements.len())
        .sum();
    
    // Should have removed unreachable blocks and/or dead assignments
    assert!(optimized_block_count < original_block_count || 
            optimized_statement_count < original_statement_count);
}

#[test]
fn test_common_subexpression_elimination() {
    let mut function = create_cse_test_function();
    let mut pass = CommonSubexpressionEliminationPass::new();
    
    // Run CSE
    let changed = pass.run_on_function(&mut function).unwrap();
    assert!(changed);
    
    // Check that the second addition was replaced with a copy
    let block = function.basic_blocks.values().next().unwrap();
    let last_assignment = &block.statements[3];
    
    if let Statement::Assign { rvalue, .. } = last_assignment {
        match rvalue {
            Rvalue::Use(Operand::Copy(_)) => {
                // Good - the common subexpression was eliminated
            }
            _ => panic!("Expected copy after CSE, found: {:?}", rvalue),
        }
    } else {
        panic!("Expected assignment statement");
    }
}

#[test]
fn test_optimization_manager_default_pipeline() {
    let mut manager = OptimizationManager::create_default_pipeline();
    let mut program = Program {
        functions: HashMap::new(),
        global_constants: HashMap::new(),
        external_functions: HashMap::new(),
        type_definitions: HashMap::new(),
    };
    
    // Add test functions to the program
    program.functions.insert("constant_test".to_string(), create_constant_folding_test_function());
    program.functions.insert("dead_code_test".to_string(), create_dead_code_test_function());
    program.functions.insert("cse_test".to_string(), create_cse_test_function());
    
    // Run the complete optimization pipeline
    assert!(manager.optimize_program(&mut program).is_ok());
    
    // Verify functions still exist after optimization
    assert!(program.functions.contains_key("constant_test"));
    assert!(program.functions.contains_key("dead_code_test"));
    assert!(program.functions.contains_key("cse_test"));
}

#[test]
fn test_optimization_manager_custom_pipeline() {
    let mut manager = OptimizationManager::new();
    manager.set_max_iterations(5);
    
    // Add only constant folding pass
    manager.add_pass(Box::new(ConstantFoldingPass::new()));
    
    let mut function = create_constant_folding_test_function();
    
    assert!(manager.optimize_function(&mut function).is_ok());
    
    // Function should still be valid
    assert_eq!(function.name, "constant_test");
}

#[test]
fn test_multiple_optimization_passes() {
    let mut function = create_constant_folding_test_function();
    
    // Run constant folding first
    let mut cf_pass = ConstantFoldingPass::new();
    let cf_changed = cf_pass.run_on_function(&mut function).unwrap();
    
    // Then run dead code elimination
    let mut dce_pass = DeadCodeEliminationPass::new();
    let dce_changed = dce_pass.run_on_function(&mut function).unwrap();
    
    // At least one pass should have made changes
    assert!(cf_changed || dce_changed);
}

/// Helper function to count binary operations in a function
fn count_binary_operations(function: &Function) -> usize {
    let mut count = 0;
    
    for block in function.basic_blocks.values() {
        for statement in &block.statements {
            if let Statement::Assign { rvalue: Rvalue::BinaryOp { .. }, .. } = statement {
                count += 1;
            }
        }
    }
    
    count
}

/// Helper function to count constant values in a function
fn count_constants(function: &Function) -> usize {
    let mut count = 0;
    
    for block in function.basic_blocks.values() {
        for statement in &block.statements {
            if let Statement::Assign { rvalue: Rvalue::Use(Operand::Constant(_)), .. } = statement {
                count += 1;
            }
        }
    }
    
    count
}
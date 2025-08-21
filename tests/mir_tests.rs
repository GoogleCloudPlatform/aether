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

//! Integration tests for MIR

use aether::mir::{*, cfg};
use aether::mir::dataflow::{run_analysis, LivenessAnalysis};
use aether::mir::validation::Validator;
use aether::types::Type;
use aether::ast::PrimitiveType;
use aether::error::SourceLocation;

#[test]
fn test_mir_builder() {
    let mut builder = Builder::new();
    
    // Build a function that computes factorial
    builder.start_function(
        "factorial".to_string(),
        vec![("n".to_string(), Type::primitive(PrimitiveType::Integer))],
        Type::primitive(PrimitiveType::Integer),
    );
    
    // Create locals
    let n = 0; // Parameter
    let result = builder.new_local(Type::primitive(PrimitiveType::Integer), true);
    let temp = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    
    // Entry block: initialize result = 1
    builder.push_statement(Statement::Assign {
        place: Place { local: result, projection: vec![] },
        rvalue: Rvalue::Use(Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(1),
        })),
        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
    });
    
    // Create loop blocks
    let loop_head = builder.new_block();
    let loop_body = builder.new_block();
    let loop_end = builder.new_block();
    
    // Jump to loop
    builder.set_terminator(Terminator::Goto { target: loop_head });
    
    // Loop head: check n > 0
    builder.switch_to_block(loop_head);
    let zero = Operand::Constant(Constant {
        ty: Type::primitive(PrimitiveType::Integer),
        value: ConstantValue::Integer(0),
    });
    
    // temp = n > 0
    builder.push_statement(Statement::Assign {
        place: Place { local: temp, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Gt,
            left: Operand::Copy(Place { local: n, projection: vec![] }),
            right: zero,
        },
        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
    });
    
    builder.set_terminator(Terminator::SwitchInt {
        discriminant: Operand::Copy(Place { local: temp, projection: vec![] }),
        switch_ty: Type::primitive(PrimitiveType::Boolean),
        targets: SwitchTargets {
            values: vec![1],
            targets: vec![loop_body],
            otherwise: loop_end,
        },
    });
    
    // Loop body: result = result * n; n = n - 1
    builder.switch_to_block(loop_body);
    
    // result = result * n
    let temp2 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    builder.push_statement(Statement::Assign {
        place: Place { local: temp2, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Mul,
            left: Operand::Copy(Place { local: result, projection: vec![] }),
            right: Operand::Copy(Place { local: n, projection: vec![] }),
        },
        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
    });
    
    builder.push_statement(Statement::Assign {
        place: Place { local: result, projection: vec![] },
        rvalue: Rvalue::Use(Operand::Copy(Place { local: temp2, projection: vec![] })),
        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
    });
    
    // n = n - 1
    builder.push_statement(Statement::Assign {
        place: Place { local: n, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Sub,
            left: Operand::Copy(Place { local: n, projection: vec![] }),
            right: Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(1),
            }),
        },
        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
    });
    
    builder.set_terminator(Terminator::Goto { target: loop_head });
    
    // Loop end: return result
    builder.switch_to_block(loop_end);
    builder.set_terminator(Terminator::Return);
    
    let function = builder.finish_function();
    
    // Validate the function
    let mut validator = Validator::new();
    validator.validate_function(&function).expect("Function should be valid");
    
    // Check basic properties
    assert_eq!(function.name, "factorial");
    assert_eq!(function.parameters.len(), 1);
    assert_eq!(function.basic_blocks.len(), 4);
    
    // Test CFG analysis
    let preds = cfg::predecessors(&function, loop_head);
    assert_eq!(preds.len(), 2); // From entry and loop body
    
    let succs = cfg::successors(&function.basic_blocks[&loop_head]);
    assert_eq!(succs.len(), 2); // To loop body and loop end
}

#[test]
fn test_liveness_analysis() {
    let mut builder = Builder::new();
    
    builder.start_function(
        "simple".to_string(),
        vec![],
        Type::primitive(PrimitiveType::Integer),
    );
    
    let x = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    let y = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    let z = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    
    // x = 1
    builder.push_statement(Statement::Assign {
        place: Place { local: x, projection: vec![] },
        rvalue: Rvalue::Use(Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(1),
        })),
        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
    });
    
    // y = 2
    builder.push_statement(Statement::Assign {
        place: Place { local: y, projection: vec![] },
        rvalue: Rvalue::Use(Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(2),
        })),
        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
    });
    
    // z = x + y
    builder.push_statement(Statement::Assign {
        place: Place { local: z, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Add,
            left: Operand::Copy(Place { local: x, projection: vec![] }),
            right: Operand::Copy(Place { local: y, projection: vec![] }),
        },
        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
    });
    
    // return z
    builder.set_terminator(Terminator::Return);
    
    let function = builder.finish_function();
    
    // Run liveness analysis
    let analysis = LivenessAnalysis;
    let results = run_analysis(&function, analysis);
    
    // After z = x + y, both x and y should be dead
    // Only z should be live (needed for return)
    assert!(!results.facts.is_empty());
}

#[test]
fn test_mir_display() {
    let mut builder = Builder::new();
    
    builder.start_function(
        "add".to_string(),
        vec![
            ("a".to_string(), Type::primitive(PrimitiveType::Integer)),
            ("b".to_string(), Type::primitive(PrimitiveType::Integer)),
        ],
        Type::primitive(PrimitiveType::Integer),
    );
    
    let result = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
    
    // result = a + b
    builder.push_statement(Statement::Assign {
        place: Place { local: result, projection: vec![] },
        rvalue: Rvalue::BinaryOp {
            op: BinOp::Add,
            left: Operand::Copy(Place { local: 0, projection: vec![] }),
            right: Operand::Copy(Place { local: 1, projection: vec![] }),
        },
        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
    });
    
    builder.set_terminator(Terminator::Return);
    
    let function = builder.finish_function();
    
    // Test display
    let output = format!("{}", function);
    assert!(output.contains("fn add"));
    assert!(output.contains("bb0:"));
}
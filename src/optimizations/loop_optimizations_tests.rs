use super::*;
use crate::ast::PrimitiveType;
use crate::error::SourceLocation;
use crate::mir::{
    Builder, Constant, ConstantValue, Operand, Place, Rvalue, SourceInfo, Statement,
};
use crate::types::Type;

#[test]
fn test_loop_optimization_pass() {
    let pass = LoopOptimizationPass::new();
    assert_eq!(pass.name(), "AdvancedLoopOptimizations");
    assert!(pass.loops.is_empty());
}

#[test]
fn test_loop_info_creation() {
    let loop_info = LoopInfo {
        header: 0,
        preheader: Some(1),
        blocks: [0, 2, 3].iter().cloned().collect(),
        exits: [4].iter().cloned().collect(),
        back_edges: vec![(3, 0)],
        depth: 1,
        parent: None,
        children: Vec::new(),
        bounds: None,
        iteration_count: Some(10),
    };

    assert_eq!(loop_info.header, 0);
    assert_eq!(loop_info.preheader, Some(1));
    assert_eq!(loop_info.blocks.len(), 3);
    assert_eq!(loop_info.iteration_count, Some(10));
}

#[test]
fn test_basic_induction_variable() {
    let basic_iv = BasicInductionVar {
        variable: Place {
            local: 0,
            projection: vec![],
        },
        initial_value: Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(0),
        }),
        step: 1,
        increment_block: 0,
        increment_statement: 2,
    };

    assert_eq!(basic_iv.step, 1);
    assert_eq!(basic_iv.increment_block, 0);
}

#[test]
fn test_derived_induction_variable() {
    let derived_iv = DerivedInductionVar {
        variable: Place {
            local: 1,
            projection: vec![],
        },
        base: Place {
            local: 0,
            projection: vec![],
        },
        multiplier: 4,
        offset: 10,
    };

    assert_eq!(derived_iv.multiplier, 4);
    assert_eq!(derived_iv.offset, 10);
}

#[test]
fn test_dependence_analysis() {
    let dep = Dependence {
        source: StatementRef {
            block: 0,
            statement: 1,
        },
        sink: StatementRef {
            block: 0,
            statement: 3,
        },
        distance: vec![2],
        direction: vec![DependenceDirection::Greater],
        dep_type: DependenceType::Flow,
    };

    assert_eq!(dep.source.block, 0);
    assert_eq!(dep.sink.statement, 3);
    assert_eq!(dep.dep_type, DependenceType::Flow);
}

#[test]
fn test_invariant_statement() {
    let invariant_stmt = InvariantStatement {
        block: 0,
        statement_index: 2,
        statement: Statement::Nop,
        safe_to_hoist: true,
        hoist_profit: 15.5,
    };

    assert_eq!(invariant_stmt.block, 0);
    assert!(invariant_stmt.safe_to_hoist);
    assert_eq!(invariant_stmt.hoist_profit, 15.5);
}

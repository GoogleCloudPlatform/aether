use super::*;
use crate::ast::PrimitiveType;
use crate::mir::{Builder, Program};
use crate::types::Type;
use std::collections::HashMap;

#[test]
fn test_interprocedural_analysis_pass() {
    let pass = InterproceduralAnalysisPass::new();
    assert_eq!(pass.name(), "InterproceduralAnalysis");
    assert!(pass.call_graph.callees.is_empty());
}

#[test]
fn test_call_graph_building() {
    let mut pass = InterproceduralAnalysisPass::new();
    let program = create_test_program();

    assert!(pass.build_call_graph(&program).is_ok());

    // Should have initialized call graph for all functions
    for function_name in program.functions.keys() {
        assert!(pass.call_graph.callees.contains_key(function_name));
        assert!(pass.call_graph.callers.contains_key(function_name));
    }
}

#[test]
fn test_side_effect_analysis() {
    let mut pass = InterproceduralAnalysisPass::new();
    let program = create_test_program();

    pass.build_call_graph(&program).unwrap();
    let summaries = pass.analyze_side_effects(&program).unwrap();

    // Should have created summaries for all functions
    for function_name in program.functions.keys() {
        assert!(summaries.contains_key(function_name));
    }
}

#[test]
fn test_function_summary() {
    let summary = FunctionSummary {
        name: "test_function".to_string(),
        side_effects: SideEffectSummary {
            reads_memory: true,
            writes_memory: false,
            performs_io: false,
            may_throw: false,
            calls_functions: false,
        },
        escaping_parameters: HashSet::new(),
        reads_globals: HashSet::new(),
        modifies_globals: HashSet::new(),
        calls: HashSet::new(),
        may_not_terminate: false,
        is_recursive: false,
    };

    assert_eq!(summary.name, "test_function");
    assert!(summary.side_effects.reads_memory);
    assert!(!summary.side_effects.writes_memory);
    assert!(!summary.is_recursive);
}

#[test]
fn test_abstract_location() {
    let param_loc = AbstractLocation::Parameter("function".to_string(), 0);
    let global_loc = AbstractLocation::Global("global_var".to_string());
    let unknown_loc = AbstractLocation::Unknown;

    assert_ne!(param_loc, global_loc);
    assert_ne!(param_loc, unknown_loc);
}

fn create_test_program() -> Program {
    let mut program = Program {
        functions: HashMap::new(),
        global_constants: HashMap::new(),
        external_functions: HashMap::new(),
        type_definitions: HashMap::new(),
    };

    // Create a simple test function
    let mut builder = Builder::new();
    builder.start_function(
        "test_function".to_string(),
        vec![],
        Type::primitive(PrimitiveType::Integer),
    );
    let function = builder.finish_function();

    program
        .functions
        .insert("test_function".to_string(), function);
    program
}

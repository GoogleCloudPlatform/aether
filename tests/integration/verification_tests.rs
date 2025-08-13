//! Enhanced verification system integration tests (Phase 2)

// Access utils from parent directory since we're in integration subdir
#[path = "../utils/mod.rs"]
mod utils;

use utils::{
    compiler_wrapper::TestCompiler,
    assertions::*,
};

#[test]
fn test_basic_contract_verification() {
    let compiler = TestCompiler::new("basic_contracts");
    
    let source = r#"
(DEFINE_MODULE basic_contracts
  (DEFINE_FUNCTION
    (NAME "safe_divide")
    (ACCEPTS_PARAMETER (NAME "numerator") (TYPE FLOAT))
    (ACCEPTS_PARAMETER (NAME "denominator") (TYPE FLOAT))
    (RETURNS (TYPE FLOAT))
    (INTENT "Performs division with guarantee against division by zero")
    (PRECONDITION 
      (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "denominator") (FLOAT_LITERAL 0.0))
      (FAILURE_ACTION THROW_EXCEPTION)
      (PROOF_HINT "denominator != 0 is checked before division"))
    (POSTCONDITION
      (PREDICATE_EQUALS 
        (VARIABLE_REFERENCE "RETURNED_VALUE")
        (EXPRESSION_DIVIDE (VARIABLE_REFERENCE "numerator") (VARIABLE_REFERENCE "denominator")))
      (PROOF_HINT "Result is mathematically correct division"))
    (BODY
      (RETURN_VALUE 
        (EXPRESSION_DIVIDE (VARIABLE_REFERENCE "numerator") (VARIABLE_REFERENCE "denominator")))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "safe_divide"
          (ARGUMENTS (FLOAT_LITERAL 10.0) (FLOAT_LITERAL 2.0)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Result: %.2f\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "basic_contracts.aether");
    assert_compile_and_execute(&result, "Result: 5.00", "Basic contract verification");
}

#[test]
fn test_contract_violation_detection() {
    let compiler = TestCompiler::new("contract_violation");
    
    let source = r#"
(DEFINE_MODULE contract_violation
  (DEFINE_FUNCTION
    (NAME "safe_array_access")
    (ACCEPTS_PARAMETER (NAME "array") (TYPE (ARRAY INT 10)))
    (ACCEPTS_PARAMETER (NAME "index") (TYPE INT))
    (RETURNS (TYPE INT))
    (PRECONDITION
      (LOGICAL_AND
        (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "index") (INTEGER_LITERAL 0))
        (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "index") (INTEGER_LITERAL 10)))
      (PROOF_HINT "Index must be within array bounds"))
    (BODY
      (RETURN_VALUE (ARRAY_ACCESS (VARIABLE_REFERENCE "array") (VARIABLE_REFERENCE "index")))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "arr") 
        (INITIAL_VALUE (ARRAY_LITERAL (TYPE (ARRAY INT 10)) 1 2 3 4 5 6 7 8 9 10)))
      
      // This should trigger a contract violation warning/error
      (DECLARE_VARIABLE (NAME "invalid_access")
        (INITIAL_VALUE (CALL_FUNCTION "safe_array_access"
          (ARGUMENTS (VARIABLE_REFERENCE "arr") (INTEGER_LITERAL 15)))))
      
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "contract_violation.aether");
    // Should either fail compilation or issue a strong warning
    if result.is_success() {
        assert_warning_contains(&result, "contract violation", "Contract violation warning");
    } else {
        assert_compilation_error(&result, "precondition", "Contract violation error");
    }
}

#[test]
fn test_contract_propagation() {
    let compiler = TestCompiler::new("contract_propagation");
    
    let source = r#"
(DEFINE_MODULE contract_propagation
  (DEFINE_FUNCTION
    (NAME "validate_positive")
    (ACCEPTS_PARAMETER (NAME "value") (TYPE INT))
    (RETURNS (TYPE BOOL))
    (POSTCONDITION
      (LOGICAL_IMPLICATION
        (VARIABLE_REFERENCE "RETURNED_VALUE")
        (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "value") (INTEGER_LITERAL 0)))
      (PROOF_HINT "If result is true, value must be positive"))
    (BODY
      (RETURN_VALUE (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "value") (INTEGER_LITERAL 0)))))
  
  (DEFINE_FUNCTION
    (NAME "safe_sqrt")
    (ACCEPTS_PARAMETER (NAME "input") (TYPE FLOAT))
    (RETURNS (TYPE FLOAT))
    (PRECONDITION
      (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "input") (FLOAT_LITERAL 0.0))
      (PROOF_HINT "Square root requires non-negative input"))
    (BODY
      (RETURN_VALUE (CALL_FUNCTION "sqrt" (ARGUMENTS (VARIABLE_REFERENCE "input"))))))
  
  (DEFINE_FUNCTION
    (NAME "validated_sqrt")
    (ACCEPTS_PARAMETER (NAME "value") (TYPE INT))
    (RETURNS (TYPE FLOAT))
    (INTENT "Compute square root after validation")
    (BODY
      (IF_CONDITION
        (CALL_FUNCTION "validate_positive" (ARGUMENTS (VARIABLE_REFERENCE "value")))
        (THEN_EXECUTE
          // Contract propagation should ensure this is safe
          (RETURN_VALUE (CALL_FUNCTION "safe_sqrt" 
            (ARGUMENTS (CAST_TO_FLOAT (VARIABLE_REFERENCE "value"))))))
        (ELSE_EXECUTE
          (RETURN_VALUE (FLOAT_LITERAL -1.0))))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "validated_sqrt" (ARGUMENTS (INTEGER_LITERAL 16)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Sqrt result: %.2f\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "contract_propagation.aether");
    assert_compile_and_execute(&result, "Sqrt result: 4.00", "Contract propagation");
}

#[test]
fn test_proof_obligations_generation() {
    let compiler = TestCompiler::new("proof_obligations");
    
    let source = r#"
(DEFINE_MODULE proof_obligations
  (DEFINE_FUNCTION
    (NAME "factorial")
    (ACCEPTS_PARAMETER (NAME "n") (TYPE INT))
    (RETURNS (TYPE INT))
    (INTENT "Calculate factorial with overflow protection")
    (PRECONDITION
      (LOGICAL_AND
        (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 0))
        (PREDICATE_LESS_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 12)))
      (PROOF_HINT "n must be in range [0, 12] to prevent overflow"))
    (POSTCONDITION
      (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "RETURNED_VALUE") (INTEGER_LITERAL 0))
      (PROOF_HINT "Factorial is always positive for valid inputs"))
    (INVARIANT
      (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 0))
      (PROOF_HINT "n remains non-negative throughout recursion"))
    (DECREASES (VARIABLE_REFERENCE "n")
      (PROOF_HINT "n decreases with each recursive call, ensuring termination"))
    (BODY
      (IF_CONDITION
        (PREDICATE_LESS_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 1))
        (THEN_EXECUTE
          (RETURN_VALUE (INTEGER_LITERAL 1)))
        (ELSE_EXECUTE
          (RETURN_VALUE 
            (EXPRESSION_MULTIPLY 
              (VARIABLE_REFERENCE "n")
              (CALL_FUNCTION "factorial" 
                (ARGUMENTS (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 1))))))))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "fact5")
        (INITIAL_VALUE (CALL_FUNCTION "factorial" (ARGUMENTS (INTEGER_LITERAL 5)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "5! = %d\n") (VARIABLE_REFERENCE "fact5")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "proof_obligations.aether");
    assert_compile_and_execute(&result, "5! = 120", "Proof obligations generation");
}

#[test]
fn test_behavioral_specification_validation() {
    let compiler = TestCompiler::new("behavioral_specs");
    
    let source = r#"
(DEFINE_MODULE behavioral_specs
  (DEFINE_FUNCTION
    (NAME "pure_calculation")
    (ACCEPTS_PARAMETER (NAME "x") (TYPE INT))
    (ACCEPTS_PARAMETER (NAME "y") (TYPE INT))
    (RETURNS (TYPE INT))
    (BEHAVIORAL_SPEC
      (IDEMPOTENT TRUE)
      (PURE TRUE)
      (SIDE_EFFECTS NONE)
      (DETERMINISTIC TRUE)
      (THREAD_SAFE TRUE))
    (INTENT "Pure mathematical calculation with no side effects")
    (BODY
      (RETURN_VALUE 
        (EXPRESSION_ADD 
          (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "x") (VARIABLE_REFERENCE "x"))
          (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "y") (VARIABLE_REFERENCE "y"))))))
  
  (DEFINE_FUNCTION
    (NAME "impure_logging")
    (ACCEPTS_PARAMETER (NAME "message") (TYPE STRING))
    (RETURNS (TYPE VOID))
    (BEHAVIORAL_SPEC
      (IDEMPOTENT FALSE)
      (PURE FALSE)
      (SIDE_EFFECTS (MODIFIES "log_file") (SENDS "debug_output"))
      (DETERMINISTIC FALSE)
      (THREAD_SAFE FALSE))
    (INTENT "Log message with side effects")
    (BODY
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "LOG: %s\n") (VARIABLE_REFERENCE "message")))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "pure_calculation"
          (ARGUMENTS (INTEGER_LITERAL 3) (INTEGER_LITERAL 4)))))
      
      (CALL_FUNCTION "impure_logging" 
        (ARGUMENTS (STRING_LITERAL "Calculation completed")))
      
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Result: %d\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "behavioral_specs.aether");
    assert_compile_and_execute(&result, "Result: 25", "Behavioral specification validation");
    
    let execution = result.execute();
    assert_output_contains(&execution, "LOG: Calculation completed", "Side effect validation");
}

#[test]
fn test_smt_solver_integration() {
    let compiler = TestCompiler::new("smt_solver");
    
    let source = r#"
(DEFINE_MODULE smt_solver
  (DEFINE_FUNCTION
    (NAME "binary_search")
    (ACCEPTS_PARAMETER (NAME "array") (TYPE (ARRAY INT 10)))
    (ACCEPTS_PARAMETER (NAME "target") (TYPE INT))
    (ACCEPTS_PARAMETER (NAME "size") (TYPE INT))
    (RETURNS (TYPE INT))
    (INTENT "Binary search in sorted array")
    (PRECONDITION
      (FORALL (VARIABLE "i") (RANGE 0 (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE "size") (INTEGER_LITERAL 1)))
        (PREDICATE_LESS_THAN_OR_EQUAL_TO 
          (ARRAY_ACCESS (VARIABLE_REFERENCE "array") (VARIABLE "i"))
          (ARRAY_ACCESS (VARIABLE_REFERENCE "array") (EXPRESSION_ADD (VARIABLE "i") (INTEGER_LITERAL 1)))))
      (PROOF_HINT "Array must be sorted in ascending order")
      (VERIFICATION_METHOD SMT_SOLVER))
    (POSTCONDITION
      (LOGICAL_OR
        (LOGICAL_AND
          (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "RETURNED_VALUE") (INTEGER_LITERAL 0))
          (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "RETURNED_VALUE") (VARIABLE_REFERENCE "size"))
          (PREDICATE_EQUALS 
            (ARRAY_ACCESS (VARIABLE_REFERENCE "array") (VARIABLE_REFERENCE "RETURNED_VALUE"))
            (VARIABLE_REFERENCE "target")))
        (PREDICATE_EQUALS (VARIABLE_REFERENCE "RETURNED_VALUE") (INTEGER_LITERAL -1)))
      (PROOF_HINT "Returns valid index of target or -1 if not found")
      (VERIFICATION_METHOD SMT_SOLVER))
    (BODY
      (DECLARE_VARIABLE (NAME "left") (INITIAL_VALUE (INTEGER_LITERAL 0)))
      (DECLARE_VARIABLE (NAME "right") (INITIAL_VALUE (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE "size") (INTEGER_LITERAL 1))))
      
      (WHILE_LOOP
        (CONDITION (PREDICATE_LESS_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "left") (VARIABLE_REFERENCE "right")))
        (INVARIANT
          (LOGICAL_AND
            (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "left") (INTEGER_LITERAL 0))
            (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "right") (VARIABLE_REFERENCE "size")))
          (PROOF_HINT "left and right remain within array bounds"))
        (BODY
          (DECLARE_VARIABLE (NAME "mid")
            (INITIAL_VALUE (EXPRESSION_DIVIDE 
              (EXPRESSION_ADD (VARIABLE_REFERENCE "left") (VARIABLE_REFERENCE "right"))
              (INTEGER_LITERAL 2))))
          
          (IF_CONDITION
            (PREDICATE_EQUALS 
              (ARRAY_ACCESS (VARIABLE_REFERENCE "array") (VARIABLE_REFERENCE "mid"))
              (VARIABLE_REFERENCE "target"))
            (THEN_EXECUTE
              (RETURN_VALUE (VARIABLE_REFERENCE "mid")))
            (ELSE_EXECUTE
              (IF_CONDITION
                (PREDICATE_LESS_THAN 
                  (ARRAY_ACCESS (VARIABLE_REFERENCE "array") (VARIABLE_REFERENCE "mid"))
                  (VARIABLE_REFERENCE "target"))
                (THEN_EXECUTE
                  (ASSIGN (TARGET (VARIABLE_REFERENCE "left"))
                          (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "mid") (INTEGER_LITERAL 1)))))
                (ELSE_EXECUTE
                  (ASSIGN (TARGET (VARIABLE_REFERENCE "right"))
                          (SOURCE (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE "mid") (INTEGER_LITERAL 1))))))))))
      
      (RETURN_VALUE (INTEGER_LITERAL -1))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "sorted_array")
        (INITIAL_VALUE (ARRAY_LITERAL (TYPE (ARRAY INT 10)) 1 3 5 7 9 11 13 15 17 19)))
      
      (DECLARE_VARIABLE (NAME "found_index")
        (INITIAL_VALUE (CALL_FUNCTION "binary_search"
          (ARGUMENTS (VARIABLE_REFERENCE "sorted_array") (INTEGER_LITERAL 7) (INTEGER_LITERAL 10)))))
      
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Found at index: %d\n") (VARIABLE_REFERENCE "found_index")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "smt_solver.aether");
    assert_compile_and_execute(&result, "Found at index: 3", "SMT solver integration");
}

#[test]
fn test_intent_mismatch_detection() {
    let compiler = TestCompiler::new("intent_mismatch");
    
    let source = r#"
(DEFINE_MODULE intent_mismatch
  // Function claims to calculate average but actually calculates sum
  (DEFINE_FUNCTION
    (NAME "calculate_average")
    (ACCEPTS_PARAMETER (NAME "a") (TYPE INT))
    (ACCEPTS_PARAMETER (NAME "b") (TYPE INT))
    (RETURNS (TYPE INT))
    (INTENT "Calculate the average of two numbers")
    (BODY
      // This actually calculates sum, not average - should trigger intent mismatch
      (RETURN_VALUE (EXPRESSION_ADD (VARIABLE_REFERENCE "a") (VARIABLE_REFERENCE "b")))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "avg")
        (INITIAL_VALUE (CALL_FUNCTION "calculate_average"
          (ARGUMENTS (INTEGER_LITERAL 10) (INTEGER_LITERAL 20)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Average: %d\n") (VARIABLE_REFERENCE "avg")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "intent_mismatch.aether");
    
    // Should compile but issue intent mismatch warning
    if result.is_success() {
        assert_warning_contains(&result, "intent mismatch", "Intent mismatch warning");
    }
    
    // Program should still execute (though with wrong behavior)
    let execution = result.execute();
    if execution.is_success() {
        // This shows the mismatch - returns 30 (sum) instead of 15 (average)
        assert_output_contains(&execution, "Average: 30", "Intent mismatch execution");
    }
}
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

//! Integration tests for memory management system

use aether::lexer::Lexer;
use aether::parser::Parser;
use aether::semantic::SemanticAnalyzer;

// Commented out until function body parsing is implemented
// #[test]
// fn test_memory_analysis_for_simple_function() {
//     let source = r#"
// (DEFINE_MODULE
//     (NAME 'example')
//     (INTENT "Test module for memory analysis")
//     (CONTENT
//         (DEFINE_FUNCTION
//             (NAME 'add')
//             (INTENT "Add two numbers")
//             (PARAMETERS
//                 (PARAMETER
//                     (NAME 'x')
//                     (TYPE INTEGER)
//                 )
//                 (PARAMETER
//                     (NAME 'y')
//                     (TYPE INTEGER)
//                 )
//             )
//             (RETURNS INTEGER)
//             (BODY
//                 (RETURN (ADD 'x' 'y'))
//             )
//         )
//     )
// )
//     "#;
    
//     let mut lexer = Lexer::new(source, "test.aether".to_string());
//     let tokens = lexer.tokenize().expect("Tokenization failed");
//     let mut parser = Parser::new(tokens);
//     let program = parser.parse_program().expect("Parsing failed");
    
//     let mut analyzer = SemanticAnalyzer::new();
//     analyzer.analyze_program(&program).expect("Semantic analysis failed");
    
//     // Test passes if no panic occurs - memory analysis is integrated
// }

#[test]
fn test_memory_analysis_with_constants() {
    let source = r#"
(DEFINE_MODULE
    (NAME 'memory_test')
    (INTENT "Test memory allocation for constants")
    (CONTENT
        (DECLARE_CONSTANT
            (NAME 'PI')
            (TYPE FLOAT)
            (VALUE 3.14159)
            (INTENT "Mathematical constant pi")
        )
        (DECLARE_CONSTANT
            (NAME 'MAX_SIZE')
            (TYPE INTEGER)
            (VALUE 1000)
            (INTENT "Maximum array size")
        )
    )
)
    "#;
    
    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis failed");
}

#[test]
fn test_memory_module_parsing() {
    // Simple test to ensure the memory module itself compiles and is accessible
    use aether::memory::{MemoryAnalyzer, RegionId};
    use aether::types::TypeChecker;
    use std::rc::Rc;
    use std::cell::RefCell;
    
    let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
    let mut analyzer = MemoryAnalyzer::new(type_checker);
    
    // Create a region
    let region = analyzer.create_region(None);
    assert_eq!(region, RegionId(0));
    
    // Enter the region
    analyzer.enter_region(region);
    
    // Exit the region
    analyzer.exit_region().expect("Failed to exit region");
}

#[test]
fn test_memory_allocation_strategies() {
    use aether::memory::{AllocationStrategy, RegionId};
    
    // Test that allocation strategies can be created
    let stack = AllocationStrategy::Stack;
    let region = AllocationStrategy::Region(RegionId(1));
    let ref_counted = AllocationStrategy::RefCounted;
    let linear = AllocationStrategy::Linear;
    
    // Test pattern matching
    match stack {
        AllocationStrategy::Stack => (),
        _ => panic!("Expected Stack allocation"),
    }
    
    match region {
        AllocationStrategy::Region(id) => assert_eq!(id, RegionId(1)),
        _ => panic!("Expected Region allocation"),
    }
    
    match ref_counted {
        AllocationStrategy::RefCounted => (),
        _ => panic!("Expected RefCounted allocation"),
    }
    
    match linear {
        AllocationStrategy::Linear => (),
        _ => panic!("Expected Linear allocation"),
    }
}

#[test]
fn test_ref_counted_wrapper() {
    use aether::memory::RefCounted;
    
    let rc1 = RefCounted::new(42);
    assert_eq!(rc1.strong_count(), 1);
    
    let rc2 = rc1.clone();
    assert_eq!(rc1.strong_count(), 2);
    assert_eq!(rc2.strong_count(), 2);
}

#[test]
fn test_linear_type_wrapper() {
    use aether::memory::Linear;
    
    let linear = Linear::new(vec![1, 2, 3, 4, 5]);
    assert!(!linear.is_consumed());
    
    let value = linear.take();
    assert_eq!(value, vec![1, 2, 3, 4, 5]);
    // Note: We can't test is_consumed() after take() because take() consumes self
}
use aether::ast::*;
use aether::ffi::{FFIAnalyzer, FFIGenerator, FFITypeMapper};
use aether::semantic::SemanticAnalyzer;
use aether::types::TypeChecker;
use aether::error::SourceLocation;
use std::rc::Rc;
use std::cell::RefCell;

fn create_test_external_function() -> ExternalFunction {
    ExternalFunction {
        name: Identifier::new("add_numbers".to_string(), SourceLocation::unknown()),
        library: "mathlib".to_string(),
        symbol: Some("math_add".to_string()),
        parameters: vec![
            Parameter {
                name: Identifier::new("a".to_string(), SourceLocation::unknown()),
                param_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    source_location: SourceLocation::unknown(),
                }),
                intent: None,
                constraint: None,
                passing_mode: PassingMode::ByValue,
                source_location: SourceLocation::unknown(),
            },
            Parameter {
                name: Identifier::new("b".to_string(), SourceLocation::unknown()),
                param_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    source_location: SourceLocation::unknown(),
                }),
                intent: None,
                constraint: None,
                passing_mode: PassingMode::ByValue,
                source_location: SourceLocation::unknown(),
            },
        ],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: SourceLocation::unknown(),
        }),
        calling_convention: CallingConvention::C,
        thread_safe: true,
        may_block: false,
        variadic: false,
        ownership_info: None,
        source_location: SourceLocation::unknown(),
    }
}

fn create_string_external_function() -> ExternalFunction {
    ExternalFunction {
        name: Identifier::new("get_message".to_string(), SourceLocation::unknown()),
        library: "msglib".to_string(),
        symbol: None,
        parameters: vec![
            Parameter {
                name: Identifier::new("id".to_string(), SourceLocation::unknown()),
                param_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    source_location: SourceLocation::unknown(),
                }),
                intent: None,
                constraint: None,
                passing_mode: PassingMode::ByValue,
                source_location: SourceLocation::unknown(),
            },
        ],
        return_type: Box::new(TypeSpecifier::Pointer {
            target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::String,
                source_location: SourceLocation::unknown(),
            }),
            is_mutable: false,
            source_location: SourceLocation::unknown(),
        }),
        calling_convention: CallingConvention::C,
        thread_safe: false,
        may_block: true,
        variadic: false,
        ownership_info: Some(OwnershipInfo {
            ownership: Ownership::CalleeOwned,
            lifetime: Some(Lifetime::Static),
            deallocator: None,
        }),
        source_location: SourceLocation::unknown(),
    }
}

#[test]
fn test_external_function_analysis() {
    let loc = SourceLocation::unknown();
    
    let ext_func = create_test_external_function();
    let module = Module {
        name: Identifier::new("ffi_test".to_string(), loc.clone()),
        intent: Some("Test FFI functionality".to_string()),
        imports: Vec::new(),
        exports: Vec::new(),
        type_definitions: Vec::new(),
        constant_declarations: Vec::new(),
        function_definitions: Vec::new(),
        external_functions: vec![ext_func],
        source_location: loc.clone(),
    };
    
    let program = Program {
        modules: vec![module],
        source_location: loc,
    };
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("FFI analysis should succeed");
    
    let stats = analyzer.get_statistics();
    assert_eq!(stats.external_functions_analyzed, 1);
}

#[test]
fn test_pointer_ownership_requirement() {
    let loc = SourceLocation::unknown();
    
    // Create function with pointer but no ownership info
    let mut ext_func = create_test_external_function();
    ext_func.parameters[0].param_type = Box::new(TypeSpecifier::Pointer {
        target_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: loc.clone(),
        }),
        is_mutable: false,
        source_location: loc.clone(),
    });
    
    let module = Module {
        name: Identifier::new("ffi_pointer_test".to_string(), loc.clone()),
        intent: Some("Test pointer ownership requirement".to_string()),
        imports: Vec::new(),
        exports: Vec::new(),
        type_definitions: Vec::new(),
        constant_declarations: Vec::new(),
        function_definitions: Vec::new(),
        external_functions: vec![ext_func],
        source_location: loc.clone(),
    };
    
    let program = Program {
        modules: vec![module],
        source_location: loc,
    };
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.to_string().contains("ownership info")));
}

#[test]
fn test_c_header_generation() {
    let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
    let mut ffi_analyzer = FFIAnalyzer::new(type_checker);
    
    let ext_func1 = create_test_external_function();
    let ext_func2 = create_string_external_function();
    
    ffi_analyzer.analyze_external_function(&ext_func1).unwrap();
    ffi_analyzer.analyze_external_function(&ext_func2).unwrap();
    
    let header = ffi_analyzer.generate_c_header("test_module");
    
    // Check header guard
    assert!(header.contains("#ifndef TEST_MODULE_H"));
    assert!(header.contains("#define TEST_MODULE_H"));
    assert!(header.contains("#endif // TEST_MODULE_H"));
    
    // Check includes
    assert!(header.contains("#include <stdint.h>"));
    assert!(header.contains("#include <stdbool.h>"));
    
    // Check function declarations
    println!("Generated header:\n{}", header);
    assert!(header.contains("int64_t"));
    assert!(header.contains("add_numbers"));
    assert!(header.contains("get_message"));
}

#[test]
fn test_rust_bindings_generation() {
    let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
    let mut ffi_analyzer = FFIAnalyzer::new(type_checker);
    
    let ext_func = create_test_external_function();
    ffi_analyzer.analyze_external_function(&ext_func).unwrap();
    
    let generator = FFIGenerator::new(ffi_analyzer);
    let bindings = generator.generate_rust_bindings("mathlib");
    
    // Check Rust-specific imports
    assert!(bindings.contains("use std::os::raw::{c_char, c_void}"));
    assert!(bindings.contains("use std::ffi::CString"));
    
    // Check extern block
    assert!(bindings.contains("#[link(name = \"mathlib\")]"));
    assert!(bindings.contains("extern \"C\" {"));
    assert!(bindings.contains("pub fn math_add(a: i64, b: i64) -> i64;"));
}

#[test]
fn test_go_bindings_generation() {
    let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
    let mut ffi_analyzer = FFIAnalyzer::new(type_checker);
    
    let ext_func = create_test_external_function();
    ffi_analyzer.analyze_external_function(&ext_func).unwrap();
    
    let generator = FFIGenerator::new(ffi_analyzer);
    let bindings = generator.generate_go_bindings("mathlib", "math");
    
    // Check Go package and imports
    assert!(bindings.contains("package math"));
    assert!(bindings.contains("import \"C\""));
    assert!(bindings.contains("#cgo LDFLAGS: -lmathlib"));
    
    // Check function wrapper
    println!("Generated Go bindings:\n{}", bindings);
    assert!(bindings.contains("func"));
    assert!(bindings.contains("int64"));
    assert!(bindings.contains("C.math_add"));
}

#[test]
fn test_calling_convention_support() {
    let loc = SourceLocation::unknown();
    
    // Test different calling conventions
    let conventions = vec![
        CallingConvention::C,
        CallingConvention::StdCall,
        CallingConvention::FastCall,
        CallingConvention::System,
    ];
    
    for convention in conventions {
        let mut ext_func = create_test_external_function();
        ext_func.calling_convention = convention.clone();
        
        let module = Module {
            name: Identifier::new("convention_test".to_string(), loc.clone()),
            intent: Some("Test calling conventions".to_string()),
            imports: Vec::new(),
            exports: Vec::new(),
            type_definitions: Vec::new(),
            constant_declarations: Vec::new(),
            function_definitions: Vec::new(),
            external_functions: vec![ext_func],
            source_location: loc.clone(),
        };
        
        let program = Program {
            modules: vec![module],
            source_location: loc.clone(),
        };
        
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze_program(&program).unwrap_or_else(|_| panic!("Should handle {:?} calling convention", convention));
    }
}

#[test]
fn test_complex_type_mapping() {
    let mapper = FFITypeMapper::new();
    let type_checker = TypeChecker::new();
    
    // Test array type mapping
    let array_type = type_checker.ast_type_to_type(&TypeSpecifier::Array {
        element_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Float,
            source_location: SourceLocation::unknown(),
        }),
        size: None,
        source_location: SourceLocation::unknown(),
    }).unwrap();
    
    assert_eq!(mapper.map_to_c_type(&array_type).unwrap(), "double*");
    assert_eq!(mapper.map_to_rust_type(&array_type).unwrap(), "*const f64");
    assert_eq!(mapper.map_to_go_type(&array_type).unwrap(), "*float64");
}
use super::*;
use crate::error::SourceLocation;

fn create_test_external_function() -> ExternalFunction {
    ExternalFunction {
        name: Identifier::new("test_add".to_string(), SourceLocation::unknown()),
        library: "testlib".to_string(),
        symbol: Some("test_add_impl".to_string()),
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

#[test]
fn test_ffi_type_mapper() {
    let mapper = FFITypeMapper::new();

    // Test C mappings
    let int_type = Type::primitive(PrimitiveType::Integer);
    assert_eq!(mapper.map_to_c_type(&int_type).unwrap(), "int64_t");

    let float_type = Type::primitive(PrimitiveType::Float);
    assert_eq!(mapper.map_to_c_type(&float_type).unwrap(), "double");

    // Test Rust mappings
    assert_eq!(mapper.map_to_rust_type(&int_type).unwrap(), "i64");
    assert_eq!(mapper.map_to_rust_type(&float_type).unwrap(), "f64");

    // Test Go mappings
    assert_eq!(mapper.map_to_go_type(&int_type).unwrap(), "int64");
    assert_eq!(mapper.map_to_go_type(&float_type).unwrap(), "float64");
}

#[test]
fn test_pointer_type_mapping() {
    let mapper = FFITypeMapper::new();

    let ptr_type = Type::pointer(Type::primitive(PrimitiveType::Integer), false);
    assert_eq!(mapper.map_to_c_type(&ptr_type).unwrap(), "const int64_t*");
    assert_eq!(mapper.map_to_rust_type(&ptr_type).unwrap(), "*const i64");

    let mut_ptr_type = Type::pointer(Type::primitive(PrimitiveType::Integer), true);
    assert_eq!(mapper.map_to_c_type(&mut_ptr_type).unwrap(), "int64_t*");
    assert_eq!(mapper.map_to_rust_type(&mut_ptr_type).unwrap(), "*mut i64");
}

#[test]
fn test_ffi_analyzer() {
    let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
    let mut analyzer = FFIAnalyzer::new(type_checker);

    let ext_func = create_test_external_function();
    assert!(analyzer.analyze_external_function(&ext_func).is_ok());

    assert_eq!(analyzer.get_external_functions().len(), 1);
    assert!(analyzer.get_external_functions().contains_key("test_add"));
}

#[test]
fn test_c_header_generation() {
    let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
    let mut analyzer = FFIAnalyzer::new(type_checker);

    let ext_func = create_test_external_function();
    analyzer.analyze_external_function(&ext_func).unwrap();

    let header = analyzer.generate_c_header("test_module");
    assert!(header.contains("#ifndef TEST_MODULE_H"));
    assert!(header.contains("#include <stdint.h>"));
    assert!(header.contains("int64_t test_add_impl test_add(int64_t a, int64_t b)"));
}

#[test]
fn test_callback_registry() {
    let mut registry = CallbackRegistry::new();

    let signature = FunctionSignature {
        parameters: vec![
            Type::primitive(PrimitiveType::Integer),
            Type::primitive(PrimitiveType::Float),
        ],
        return_type: Type::primitive(PrimitiveType::Boolean),
    };

    let id = registry.register_callback(
        "my_callback".to_string(),
        signature,
        CallingConvention::C,
        true,
    );

    assert_eq!(id, 1);
    assert!(registry.get_callback(id).is_some());
    assert!(registry.get_callback_by_name("my_callback").is_some());

    let callback = registry.get_callback_by_name("my_callback").unwrap();
    assert_eq!(callback.function_name, "my_callback");
    assert!(callback.thread_safe);
    assert_eq!(callback.calling_convention, CallingConvention::C);
}

#[test]
fn test_callback_c_generation() {
    let mut registry = CallbackRegistry::new();
    let type_mapper = FFITypeMapper::new();

    let signature = FunctionSignature {
        parameters: vec![Type::primitive(PrimitiveType::Integer)],
        return_type: Type::primitive(PrimitiveType::Void),
    };

    registry.register_callback(
        "test_callback".to_string(),
        signature,
        CallingConvention::C,
        true,
    );

    let declarations = registry.generate_c_callback_declarations(&type_mapper);
    assert!(declarations.contains("typedef void (*test_callback)(int64_t param0)"));
    assert!(declarations.contains("void register_test_callback(test_callback*)"));
}

#[test]
fn test_struct_handler() {
    let mut handler = FFIStructHandler::new();

    let struct_type = StructType {
        name: "Point".to_string(),
        fields: vec![
            StructField {
                name: "x".to_string(),
                field_type: Type::primitive(PrimitiveType::Float),
                offset: None,
            },
            StructField {
                name: "y".to_string(),
                field_type: Type::primitive(PrimitiveType::Float),
                offset: None,
            },
        ],
        packed: false,
        alignment: None,
    };

    handler.register_struct(struct_type);

    let c_declarations = handler.generate_c_struct_declarations();
    assert!(c_declarations.contains("typedef struct Point {"));
    assert!(c_declarations.contains("double x;"));
    assert!(c_declarations.contains("double y;"));
    assert!(c_declarations.contains("} Point;"));

    let rust_declarations = handler.generate_rust_struct_declarations();
    assert!(rust_declarations.contains("#[repr(C)]"));
    assert!(rust_declarations.contains("pub struct Point {"));
    assert!(rust_declarations.contains("pub x: f64,"));
    assert!(rust_declarations.contains("pub y: f64,"));
}

#[test]
fn test_enum_handler() {
    let mut handler = FFIStructHandler::new();

    let enum_type = EnumType {
        name: "Color".to_string(),
        variants: vec![
            EnumVariant {
                name: "Red".to_string(),
                value: Some(0),
            },
            EnumVariant {
                name: "Green".to_string(),
                value: Some(1),
            },
            EnumVariant {
                name: "Blue".to_string(),
                value: Some(2),
            },
        ],
        underlying_type: Type::primitive(PrimitiveType::Integer),
    };

    handler.register_enum(enum_type);

    let c_declarations = handler.generate_c_struct_declarations();
    assert!(c_declarations.contains("typedef enum : int64_t {"));
    assert!(c_declarations.contains("COLOR__RED = 0,"));
    assert!(c_declarations.contains("COLOR__GREEN = 1,"));
    assert!(c_declarations.contains("COLOR__BLUE = 2,"));
    assert!(c_declarations.contains("} Color;"));

    let rust_declarations = handler.generate_rust_struct_declarations();
    assert!(rust_declarations.contains("#[repr(i64)]"));
    assert!(rust_declarations.contains("pub enum Color {"));
    assert!(rust_declarations.contains("Red = 0,"));
    assert!(rust_declarations.contains("Green = 1,"));
    assert!(rust_declarations.contains("Blue = 2,"));
}

#[test]
fn test_binding_generator() {
    let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
    let mut generator = BindingGenerator::new(type_checker);

    // Add external function
    let ext_func = create_test_external_function();
    generator.add_external_function(&ext_func).unwrap();

    // Add callback
    let signature = FunctionSignature {
        parameters: vec![Type::primitive(PrimitiveType::Integer)],
        return_type: Type::primitive(PrimitiveType::Void),
    };
    generator.add_callback(
        "event_handler".to_string(),
        signature,
        CallingConvention::C,
        true,
    );

    // Add struct
    let struct_type = StructType {
        name: "TestStruct".to_string(),
        fields: vec![StructField {
            name: "value".to_string(),
            field_type: Type::primitive(PrimitiveType::Integer),
            offset: None,
        }],
        packed: false,
        alignment: None,
    };
    generator.add_struct(struct_type);

    // Generate C bindings
    let c_bindings = generator
        .generate_complete_bindings("test", TargetLanguage::C)
        .unwrap();
    assert!(c_bindings.contains("#ifndef TEST_BINDINGS_H"));
    assert!(c_bindings.contains("typedef struct TestStruct {"));
    assert!(c_bindings.contains("typedef void (*event_handler)"));

    // Generate Rust bindings
    let rust_bindings = generator
        .generate_complete_bindings("test", TargetLanguage::Rust)
        .unwrap();
    assert!(rust_bindings.contains("use std::os::raw"));
    assert!(rust_bindings.contains("#[repr(C)]"));
    assert!(rust_bindings.contains("pub type event_handler_callback"));

    // Generate Go bindings
    let go_bindings = generator
        .generate_complete_bindings("test", TargetLanguage::Go)
        .unwrap();
    assert!(go_bindings.contains("package test_bindings"));
    assert!(go_bindings.contains("import \"C\""));
}

#[test]
fn test_target_language_display() {
    assert_eq!(format!("{}", TargetLanguage::C), "C");
    assert_eq!(format!("{}", TargetLanguage::Rust), "Rust");
    assert_eq!(format!("{}", TargetLanguage::Go), "Go");
}

#[test]
fn test_rust_bindings_generation() {
    let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
    let mut analyzer = FFIAnalyzer::new(type_checker);

    let ext_func = create_test_external_function();
    analyzer.analyze_external_function(&ext_func).unwrap();

    let generator = FFIGenerator::new(analyzer);
    let bindings = generator.generate_rust_bindings("test_module");

    assert!(bindings.contains("use std::os::raw"));
    assert!(bindings.contains("#[link(name = \"test_module\")]"));
    assert!(bindings.contains("pub fn test_add_impl(a: i64, b: i64) -> i64;"));
}

#[test]
fn test_ownership_validation() {
    let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
    let mut analyzer = FFIAnalyzer::new(type_checker);

    // Create function with pointer parameter but no ownership info
    let mut ext_func = create_test_external_function();
    ext_func.parameters[0].param_type = Box::new(TypeSpecifier::Pointer {
        target_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: SourceLocation::unknown(),
        }),
        is_mutable: false,
        source_location: SourceLocation::unknown(),
    });

    // For C calling convention, ownership info is not required
    // Should succeed even without ownership info
    let result = analyzer.analyze_external_function(&ext_func);
    assert!(result.is_ok());

    // Change to System calling convention - now it should require ownership info
    ext_func.calling_convention = CallingConvention::System;
    let result = analyzer.analyze_external_function(&ext_func);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("ownership info"));

    // Add ownership info
    ext_func.ownership_info = Some(OwnershipInfo {
        ownership: Ownership::Borrowed,
        lifetime: Some(Lifetime::CallDuration),
        deallocator: None,
    });

    // Should succeed with ownership info
    assert!(analyzer.analyze_external_function(&ext_func).is_ok());
}

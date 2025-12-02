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

//! std.math - Mathematical functions and constants

use super::{create_external_function_named, create_function_stub};
use crate::ast::CallingConvention;
use crate::ast::{
    ConstantDeclaration, ExportStatement, Expression, Identifier, Module, PrimitiveType,
    TypeSpecifier,
};
use crate::error::SourceLocation;
use std::collections::HashMap;

/// Create the std.math module with mathematical functions
pub fn create_math_module() -> Module {
    let mut external_functions = HashMap::new();
    let mut functions = HashMap::new();
    let mut constants = HashMap::new();

    // Common types
    let float_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Float,
        source_location: SourceLocation::unknown(),
    };
    let int_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Integer,
        source_location: SourceLocation::unknown(),
    };
    let bool_type = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Boolean,
        source_location: SourceLocation::unknown(),
    };

    // Mathematical constants
    constants.insert(
        "PI".to_string(),
        ConstantDeclaration {
            name: Identifier::new("PI".to_string(), SourceLocation::unknown()),
            type_spec: Box::new(float_type.clone()),
            value: Box::new(Expression::FloatLiteral {
                value: std::f64::consts::PI,
                source_location: SourceLocation::unknown(),
            }),
            intent: None,
            source_location: SourceLocation::unknown(),
        },
    );

    constants.insert(
        "E".to_string(),
        ConstantDeclaration {
            name: Identifier::new("E".to_string(), SourceLocation::unknown()),
            type_spec: Box::new(float_type.clone()),
            value: Box::new(Expression::FloatLiteral {
                value: std::f64::consts::E,
                source_location: SourceLocation::unknown(),
            }),
            intent: None,
            source_location: SourceLocation::unknown(),
        },
    );

    constants.insert(
        "TAU".to_string(),
        ConstantDeclaration {
            name: Identifier::new("TAU".to_string(), SourceLocation::unknown()),
            type_spec: Box::new(float_type.clone()),
            value: Box::new(Expression::FloatLiteral {
                value: std::f64::consts::TAU,
                source_location: SourceLocation::unknown(),
            }),
            intent: None,
            source_location: SourceLocation::unknown(),
        },
    );

    constants.insert(
        "SQRT_2".to_string(),
        ConstantDeclaration {
            name: Identifier::new("SQRT_2".to_string(), SourceLocation::unknown()),
            type_spec: Box::new(float_type.clone()),
            value: Box::new(Expression::FloatLiteral {
                value: std::f64::consts::SQRT_2,
                source_location: SourceLocation::unknown(),
            }),
            intent: None,
            source_location: SourceLocation::unknown(),
        },
    );

    constants.insert(
        "LN_2".to_string(),
        ConstantDeclaration {
            name: Identifier::new("LN_2".to_string(), SourceLocation::unknown()),
            type_spec: Box::new(float_type.clone()),
            value: Box::new(Expression::FloatLiteral {
                value: std::f64::consts::LN_2,
                source_location: SourceLocation::unknown(),
            }),
            intent: None,
            source_location: SourceLocation::unknown(),
        },
    );

    constants.insert(
        "LN_10".to_string(),
        ConstantDeclaration {
            name: Identifier::new("LN_10".to_string(), SourceLocation::unknown()),
            type_spec: Box::new(float_type.clone()),
            value: Box::new(Expression::FloatLiteral {
                value: std::f64::consts::LN_10,
                source_location: SourceLocation::unknown(),
            }),
            intent: None,
            source_location: SourceLocation::unknown(),
        },
    );

    // Basic arithmetic functions (external for precision)
    external_functions.insert(
        "sqrt".to_string(),
        create_external_function_named(
            "sqrt",
            "aether_sqrt",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "pow".to_string(),
        create_external_function_named(
            "pow",
            "aether_pow",
            vec![
                ("base", float_type.clone()),
                ("exponent", float_type.clone()),
            ],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "exp".to_string(),
        create_external_function_named(
            "exp",
            "aether_exp",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "log".to_string(),
        create_external_function_named(
            "log",
            "aether_log",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "log10".to_string(),
        create_external_function_named(
            "log10",
            "aether_log10",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "log2".to_string(),
        create_external_function_named(
            "log2",
            "aether_log2",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    // Trigonometric functions
    external_functions.insert(
        "sin".to_string(),
        create_external_function_named(
            "sin",
            "aether_sin",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "cos".to_string(),
        create_external_function_named(
            "cos",
            "aether_cos",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "tan".to_string(),
        create_external_function_named(
            "tan",
            "aether_tan",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "asin".to_string(),
        create_external_function_named(
            "asin",
            "aether_asin",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "acos".to_string(),
        create_external_function_named(
            "acos",
            "aether_acos",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "atan".to_string(),
        create_external_function_named(
            "atan",
            "aether_atan",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "atan2".to_string(),
        create_external_function_named(
            "atan2",
            "aether_atan2",
            vec![("y", float_type.clone()), ("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    // Hyperbolic functions
    external_functions.insert(
        "sinh".to_string(),
        create_external_function_named(
            "sinh",
            "aether_sinh",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "cosh".to_string(),
        create_external_function_named(
            "cosh",
            "aether_cosh",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "tanh".to_string(),
        create_external_function_named(
            "tanh",
            "aether_tanh",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    // Rounding and comparison functions
    external_functions.insert(
        "floor".to_string(),
        create_external_function_named(
            "floor",
            "aether_floor",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "ceil".to_string(),
        create_external_function_named(
            "ceil",
            "aether_ceil",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "round".to_string(),
        create_external_function_named(
            "round",
            "aether_round",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "fabs".to_string(),
        create_external_function_named(
            "fabs",
            "aether_fabs",
            vec![("x", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    external_functions.insert(
        "fmod".to_string(),
        create_external_function_named(
            "fmod",
            "aether_fmod",
            vec![("x", float_type.clone()), ("y", float_type.clone())],
            float_type.clone(),
            CallingConvention::C,
        ),
    );

    // High-level utility functions (implemented in AetherScript)
    functions.insert(
        "abs".to_string(),
        create_function_stub("abs", vec![("x", float_type.clone())], float_type.clone()),
    );

    functions.insert(
        "min".to_string(),
        create_function_stub(
            "min",
            vec![("a", float_type.clone()), ("b", float_type.clone())],
            float_type.clone(),
        ),
    );

    functions.insert(
        "max".to_string(),
        create_function_stub(
            "max",
            vec![("a", float_type.clone()), ("b", float_type.clone())],
            float_type.clone(),
        ),
    );

    functions.insert(
        "clamp".to_string(),
        create_function_stub(
            "clamp",
            vec![
                ("value", float_type.clone()),
                ("min_val", float_type.clone()),
                ("max_val", float_type.clone()),
            ],
            float_type.clone(),
        ),
    );

    functions.insert(
        "lerp".to_string(),
        create_function_stub(
            "lerp",
            vec![
                ("a", float_type.clone()),
                ("b", float_type.clone()),
                ("t", float_type.clone()),
            ],
            float_type.clone(),
        ),
    );

    functions.insert(
        "radians".to_string(),
        create_function_stub(
            "radians",
            vec![("degrees", float_type.clone())],
            float_type.clone(),
        ),
    );

    functions.insert(
        "degrees".to_string(),
        create_function_stub(
            "degrees",
            vec![("radians", float_type.clone())],
            float_type.clone(),
        ),
    );

    functions.insert(
        "sign".to_string(),
        create_function_stub("sign", vec![("x", float_type.clone())], int_type.clone()),
    );

    functions.insert(
        "is_nan".to_string(),
        create_function_stub("is_nan", vec![("x", float_type.clone())], bool_type.clone()),
    );

    functions.insert(
        "is_infinite".to_string(),
        create_function_stub(
            "is_infinite",
            vec![("x", float_type.clone())],
            bool_type.clone(),
        ),
    );

    functions.insert(
        "is_finite".to_string(),
        create_function_stub(
            "is_finite",
            vec![("x", float_type.clone())],
            bool_type.clone(),
        ),
    );

    // Integer math functions
    functions.insert(
        "gcd".to_string(),
        create_function_stub(
            "gcd",
            vec![("a", int_type.clone()), ("b", int_type.clone())],
            int_type.clone(),
        ),
    );

    functions.insert(
        "lcm".to_string(),
        create_function_stub(
            "lcm",
            vec![("a", int_type.clone()), ("b", int_type.clone())],
            int_type.clone(),
        ),
    );

    functions.insert(
        "factorial".to_string(),
        create_function_stub("factorial", vec![("n", int_type.clone())], int_type.clone()),
    );

    functions.insert(
        "fibonacci".to_string(),
        create_function_stub("fibonacci", vec![("n", int_type.clone())], int_type.clone()),
    );

    Module {
        name: Identifier::new("std.math".to_string(), SourceLocation::unknown()),
        intent: Some(
            "Provides mathematical functions and constants for numerical computations".to_string(),
        ),
        imports: vec![],
        exports: vec![ExportStatement::Constant {
            name: Identifier::new("PI".to_string(), SourceLocation::unknown()),
            source_location: SourceLocation::unknown(),
        }],
        type_definitions: vec![],
        trait_definitions: vec![],
        impl_blocks: vec![],
        constant_declarations: constants.into_values().collect(),
        function_definitions: functions.into_values().collect(),
        external_functions: external_functions.into_values().collect(),
        source_location: SourceLocation::unknown(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_module_creation() {
        let module = create_math_module();

        assert_eq!(module.name.name, "std.math");
        assert!(module.intent.is_some());

        // Check constants
        assert!(module
            .constant_declarations
            .iter()
            .any(|c| c.name.name == "PI"));
        assert!(module
            .constant_declarations
            .iter()
            .any(|c| c.name.name == "E"));
        assert!(module
            .constant_declarations
            .iter()
            .any(|c| c.name.name == "TAU"));

        // Check external functions
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "sqrt"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "sin"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "cos"));
        assert!(module
            .external_functions
            .iter()
            .any(|f| f.name.name == "pow"));

        // Check high-level functions
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "abs"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "min"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "max"));
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "gcd"));

        // Check exports
        assert!(module.exports.len() > 0);
    }

    #[test]
    fn test_math_constants() {
        let module = create_math_module();

        // Check PI constant
        let pi_const = module
            .constant_declarations
            .iter()
            .find(|c| c.name.name == "PI")
            .unwrap();
        assert_eq!(pi_const.name.name, "PI");
        if let Expression::FloatLiteral { value, .. } = pi_const.value.as_ref() {
            assert!((value - std::f64::consts::PI).abs() < f64::EPSILON);
        } else {
            panic!("PI constant should be a float literal");
        }

        // Check E constant
        let e_const = module
            .constant_declarations
            .iter()
            .find(|c| c.name.name == "E")
            .unwrap();
        assert_eq!(e_const.name.name, "E");
        if let Expression::FloatLiteral { value, .. } = e_const.value.as_ref() {
            assert!((value - std::f64::consts::E).abs() < f64::EPSILON);
        } else {
            panic!("E constant should be a float literal");
        }
    }

    #[test]
    fn test_trigonometric_functions() {
        let module = create_math_module();

        // Test sin function
        let sin_func = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "sin")
            .unwrap();
        assert_eq!(sin_func.name.name, "sin");
        assert_eq!(sin_func.parameters.len(), 1);
        assert_eq!(sin_func.parameters[0].name.name, "x");

        // Test atan2 function (two parameters)
        let atan2_func = module
            .external_functions
            .iter()
            .find(|f| f.name.name == "atan2")
            .unwrap();
        assert_eq!(atan2_func.parameters.len(), 2);
        assert_eq!(atan2_func.parameters[0].name.name, "y");
        assert_eq!(atan2_func.parameters[1].name.name, "x");
    }

    #[test]
    fn test_utility_functions() {
        let module = create_math_module();

        // Test clamp function
        let clamp_func = module
            .function_definitions
            .iter()
            .find(|f| f.name.name == "clamp")
            .unwrap();
        assert_eq!(clamp_func.name.name, "clamp");
        assert_eq!(clamp_func.parameters.len(), 3);

        // Test gcd function
        let gcd_func = module
            .function_definitions
            .iter()
            .find(|f| f.name.name == "gcd")
            .unwrap();
        assert_eq!(gcd_func.parameters.len(), 2);
        assert!(matches!(
            gcd_func.return_type.as_ref(),
            TypeSpecifier::Primitive {
                type_name: crate::ast::PrimitiveType::Integer,
                ..
            }
        ));
    }
}

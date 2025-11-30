use super::*;

#[test]
fn test_primitive_types() {
    let int_type = Type::primitive(PrimitiveType::Integer);
    let float_type = Type::primitive(PrimitiveType::Float);
    let string_type = Type::primitive(PrimitiveType::String);

    assert!(int_type.is_numeric());
    assert!(int_type.is_integer());
    assert!(!int_type.is_float());

    assert!(float_type.is_numeric());
    assert!(!float_type.is_integer());
    assert!(float_type.is_float());

    assert!(!string_type.is_numeric());
    assert!(!string_type.is_integer());
    assert!(!string_type.is_float());
}

#[test]
fn test_compound_types() {
    let int_type = Type::primitive(PrimitiveType::Integer);
    let array_type = Type::array(int_type.clone(), Some(10));
    let _map_type = Type::map(Type::primitive(PrimitiveType::String), int_type.clone());
    let pointer_type = Type::pointer(int_type, false);

    assert_eq!(array_type.size_bytes(), None); // Size depends on element type
    assert_eq!(pointer_type.size_bytes(), Some(8)); // 64-bit pointer
    assert!(pointer_type.is_pointer());
}

#[test]
fn test_type_compatibility() {
    let checker = TypeChecker::new();

    let int_type = Type::primitive(PrimitiveType::Integer);
    let float_type = Type::primitive(PrimitiveType::Float);
    let string_type = Type::primitive(PrimitiveType::String);

    // Same types are compatible
    assert!(checker.types_compatible(&int_type, &int_type));

    // Numeric types are compatible
    assert!(checker.types_compatible(&int_type, &float_type));
    assert!(checker.types_compatible(&float_type, &int_type));

    // Different types are not compatible
    assert!(!checker.types_compatible(&int_type, &string_type));
}

#[test]
fn test_type_unification() {
    let mut checker = TypeChecker::new();

    let int_type = Type::primitive(PrimitiveType::Integer);
    let var_type = checker.fresh_type_var();

    // Unify type variable with concrete type
    assert!(checker.unify(&var_type, &int_type).is_ok());

    // Check that substitution was applied
    let unified = checker.apply_substitutions(&var_type);
    assert_eq!(unified, int_type);
}

#[test]
fn test_type_display() {
    let int_type = Type::primitive(PrimitiveType::Integer);
    let array_type = Type::array(int_type.clone(), Some(5));
    let map_type = Type::map(Type::primitive(PrimitiveType::String), int_type.clone());

    assert_eq!(int_type.to_string(), "Integer");
    assert_eq!(array_type.to_string(), "[Integer; 5]");
    assert_eq!(map_type.to_string(), "Map<String, Integer>");
}

#[test]
fn test_generic_types() {
    let int_type = Type::primitive(PrimitiveType::Integer);
    let string_type = Type::primitive(PrimitiveType::String);

    // Test generic type parameter
    let generic_param = Type::generic(
        "T".to_string(),
        vec![
            TypeConstraintInfo::NumericBound,
            TypeConstraintInfo::EqualityBound,
        ],
    );

    // Test generic instance
    let list_int = Type::generic_instance("List".to_string(), vec![int_type.clone()], None);

    let map_string_int = Type::generic_instance(
        "Map".to_string(),
        vec![string_type.clone(), int_type.clone()],
        None,
    );

    assert_eq!(generic_param.to_string(), "T: NumericBound + EqualityBound");
    assert_eq!(list_int.to_string(), "List<Integer>");
    assert_eq!(map_string_int.to_string(), "Map<String, Integer>");
}

#[test]
fn test_constraint_checking() {
    let checker = TypeChecker::new();
    let int_type = Type::primitive(PrimitiveType::Integer);
    let string_type = Type::primitive(PrimitiveType::String);
    let func_type = Type::function(vec![int_type.clone()], int_type.clone());

    // Test numeric constraint
    let numeric_constraints = vec![TypeConstraintInfo::NumericBound];
    assert!(checker
        .check_constraints(&int_type, &numeric_constraints)
        .is_ok());
    assert!(checker
        .check_constraints(&string_type, &numeric_constraints)
        .is_err());

    // Test equality constraint
    let equality_constraints = vec![TypeConstraintInfo::EqualityBound];
    assert!(checker
        .check_constraints(&int_type, &equality_constraints)
        .is_ok());
    assert!(checker
        .check_constraints(&string_type, &equality_constraints)
        .is_ok());
    assert!(checker
        .check_constraints(&func_type, &equality_constraints)
        .is_err());

    // Test order constraint
    let order_constraints = vec![TypeConstraintInfo::OrderBound];
    assert!(checker
        .check_constraints(&int_type, &order_constraints)
        .is_ok());
    assert!(checker
        .check_constraints(&string_type, &order_constraints)
        .is_ok());
    assert!(checker
        .check_constraints(&func_type, &order_constraints)
        .is_err());
}

#[test]
fn test_generic_instantiation() {
    let checker = TypeChecker::new();
    let int_type = Type::primitive(PrimitiveType::Integer);
    let string_type = Type::primitive(PrimitiveType::String);

    // Test successful instantiation
    let numeric_constraints = vec![vec![TypeConstraintInfo::NumericBound]];
    let result = checker.instantiate_generic("List", &[int_type.clone()], &numeric_constraints);
    assert!(result.is_ok());

    // Test constraint violation
    let result =
        checker.instantiate_generic("List", &[string_type.clone()], &numeric_constraints);
    assert!(result.is_err());

    // Test wrong number of arguments
    let result = checker.instantiate_generic(
        "List",
        &[int_type.clone(), string_type.clone()],
        &numeric_constraints,
    );
    assert!(result.is_err());
}

#[test]
fn test_generic_unification() {
    let mut checker = TypeChecker::new();
    let int_type = Type::primitive(PrimitiveType::Integer);
    let string_type = Type::primitive(PrimitiveType::String);

    let list_int1 = Type::generic_instance("List".to_string(), vec![int_type.clone()], None);
    let list_int2 = Type::generic_instance("List".to_string(), vec![int_type.clone()], None);
    let list_string =
        Type::generic_instance("List".to_string(), vec![string_type.clone()], None);

    // Same generic instances should unify
    assert!(checker.unify(&list_int1, &list_int2).is_ok());

    // Different generic instances should not unify
    assert!(checker.unify(&list_int1, &list_string).is_err());
}

#[test]
fn test_ownership_types() {
    let checker = TypeChecker::new();

    let int_type = Type::primitive(PrimitiveType::Integer);
    let owned_int = Type::owned(int_type.clone());
    let borrowed_int = Type::borrowed(int_type.clone());
    let mut_borrowed_int = Type::mutable_borrow(int_type.clone());
    let shared_int = Type::shared(int_type.clone());

    // Test ownership kind extraction
    assert_eq!(owned_int.ownership_kind(), Some(OwnershipKind::Owned));
    assert_eq!(borrowed_int.ownership_kind(), Some(OwnershipKind::Borrowed));
    assert_eq!(
        mut_borrowed_int.ownership_kind(),
        Some(OwnershipKind::MutableBorrow)
    );
    assert_eq!(shared_int.ownership_kind(), Some(OwnershipKind::Shared));
    assert_eq!(int_type.ownership_kind(), None);

    // Test base type extraction
    assert_eq!(owned_int.base_type(), &int_type);
    assert_eq!(int_type.base_type(), &int_type);

    // Test ownership checks
    assert!(owned_int.is_owned());
    assert!(!borrowed_int.is_owned());
    assert!(borrowed_int.is_borrowed());
    assert!(mut_borrowed_int.is_borrowed());
    assert!(!shared_int.is_borrowed());
}

#[test]
fn test_ownership_compatibility() {
    let checker = TypeChecker::new();

    let int_type = Type::primitive(PrimitiveType::Integer);
    let owned_int = Type::owned(int_type.clone());
    let borrowed_int = Type::borrowed(int_type.clone());
    let mut_borrowed_int = Type::mutable_borrow(int_type.clone());
    let shared_int = Type::shared(int_type.clone());

    // Owned can be borrowed (Target: Borrowed, Source: Owned)
    assert!(checker.types_compatible(&borrowed_int, &owned_int));
    assert!(checker.types_compatible(&mut_borrowed_int, &owned_int));

    // Mutable borrow can be used as immutable borrow (Target: Borrowed, Source: MutableBorrow)
    assert!(checker.types_compatible(&borrowed_int, &mut_borrowed_int));

    // Shared can be borrowed immutably (Target: Borrowed, Source: Shared)
    assert!(checker.types_compatible(&borrowed_int, &shared_int));

    // But not the other way around (Target: Owned, Source: Borrowed)
    assert!(!checker.types_compatible(&owned_int, &borrowed_int));
    assert!(!checker.types_compatible(&mut_borrowed_int, &borrowed_int));

    // Different ownership kinds don't unify unless compatible
    assert!(!checker.types_compatible(&shared_int, &owned_int));
}

#[test]
fn test_ownership_display() {
    let int_type = Type::primitive(PrimitiveType::Integer);
    let owned_int = Type::owned(int_type.clone());
    let borrowed_int = Type::borrowed(int_type.clone());
    let mut_borrowed_int = Type::mutable_borrow(int_type.clone());
    let shared_int = Type::shared(int_type.clone());

    assert_eq!(owned_int.to_string(), "^Integer");
    assert_eq!(borrowed_int.to_string(), "&Integer");
    assert_eq!(mut_borrowed_int.to_string(), "&mut Integer");
    assert_eq!(shared_int.to_string(), "~Integer");
}

#[test]
fn test_ast_to_type_ownership() {
    use crate::ast::{Identifier, TypeSpecifier};
    use crate::error::SourceLocation;

    let checker = TypeChecker::new();
    let int_spec = TypeSpecifier::Primitive {
        type_name: PrimitiveType::Integer,
        source_location: SourceLocation::unknown(),
    };

    // Test owned type conversion
    let owned_spec = TypeSpecifier::Owned {
        ownership: crate::ast::OwnershipKind::Owned,
        base_type: Box::new(int_spec.clone()),
        lifetime: None,
        source_location: SourceLocation::unknown(),
    };

    let owned_type = checker.ast_type_to_type(&owned_spec).unwrap();
    assert!(owned_type.is_owned());
    assert_eq!(owned_type.ownership_kind(), Some(OwnershipKind::Owned));

    // Test borrowed type conversion
    let borrowed_spec = TypeSpecifier::Owned {
        ownership: crate::ast::OwnershipKind::Borrowed,
        base_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::String,
            source_location: SourceLocation::unknown(),
        }),
        lifetime: None,
        source_location: SourceLocation::unknown(),
    };

    let borrowed_type = checker.ast_type_to_type(&borrowed_spec).unwrap();
    assert!(borrowed_type.is_borrowed());
    assert_eq!(
        borrowed_type.ownership_kind(),
        Some(OwnershipKind::Borrowed)
    );
}

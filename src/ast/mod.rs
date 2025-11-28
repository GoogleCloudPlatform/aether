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

//! Abstract Syntax Tree definitions for AetherScript
//!
//! Defines AST node types for all language constructs

use crate::error::SourceLocation;
use serde::{Deserialize, Serialize};

pub mod resource;

/// Visitor trait for AST traversal
pub trait ASTVisitor<T> {
    fn visit_module(&mut self, node: &Module) -> T;
    fn visit_function(&mut self, node: &Function) -> T;
    fn visit_expression(&mut self, node: &Expression) -> T;
    fn visit_statement(&mut self, node: &Statement) -> T;
    fn visit_type_definition(&mut self, node: &TypeDefinition) -> T;
}

/// Root AST node representing an entire AetherScript program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub modules: Vec<Module>,
    pub source_location: SourceLocation,
}

/// Module definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: Identifier,
    pub intent: Option<String>,
    pub imports: Vec<ImportStatement>,
    pub exports: Vec<ExportStatement>,
    pub type_definitions: Vec<TypeDefinition>,
    pub constant_declarations: Vec<ConstantDeclaration>,
    pub function_definitions: Vec<Function>,
    pub external_functions: Vec<ExternalFunction>,
    pub source_location: SourceLocation,
}

/// Import statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatement {
    pub module_name: Identifier,
    pub alias: Option<Identifier>,
    pub source_location: SourceLocation,
}

/// Export statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportStatement {
    Function {
        name: Identifier,
        source_location: SourceLocation,
    },
    Type {
        name: Identifier,
        source_location: SourceLocation,
    },
    Constant {
        name: Identifier,
        source_location: SourceLocation,
    },
}

/// Type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeDefinition {
    Structured {
        name: Identifier,
        intent: Option<String>,
        generic_parameters: Vec<GenericParameter>,
        lifetime_parameters: Vec<LifetimeParameter>,
        fields: Vec<StructField>,
        export_as: Option<String>, // For FFI
        source_location: SourceLocation,
    },
    Enumeration {
        name: Identifier,
        intent: Option<String>,
        generic_parameters: Vec<GenericParameter>,
        lifetime_parameters: Vec<LifetimeParameter>,
        variants: Vec<EnumVariant>,
        source_location: SourceLocation,
    },
    Alias {
        new_name: Identifier,
        original_type: Box<TypeSpecifier>,
        intent: Option<String>,
        generic_parameters: Vec<GenericParameter>,
        lifetime_parameters: Vec<LifetimeParameter>,
        source_location: SourceLocation,
    },
}

/// Structure field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub name: Identifier,
    pub field_type: Box<TypeSpecifier>,
    pub source_location: SourceLocation,
}

/// Enumeration variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: Identifier,
    pub associated_types: Vec<TypeSpecifier>, // Types held by the variant
    pub source_location: SourceLocation,
}

/// Type constraint for generic parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeConstraint {
    pub constraint_type: TypeConstraintKind,
    pub source_location: SourceLocation,
}

/// Kind of type constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeConstraintKind {
    /// Type must implement a trait/interface
    TraitBound { trait_name: Identifier },
    /// Type must be a subtype of another type
    SubtypeBound { parent_type: Box<TypeSpecifier> },
    /// Type must satisfy a size constraint
    SizeBound { size_expr: Box<Expression> },
    /// Type must be a numeric type
    NumericBound,
    /// Type must be an equality-comparable type
    EqualityBound,
    /// Type must be an order-comparable type
    OrderBound,
    /// Custom constraint expression
    CustomBound { constraint_expr: Box<Expression> },
}

/// Type specifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeSpecifier {
    Primitive {
        type_name: PrimitiveType,
        source_location: SourceLocation,
    },
    Named {
        name: Identifier,
        source_location: SourceLocation,
    },
    /// Generic type instantiation (e.g., List<T>, Result<T, E>)
    Generic {
        base_type: Identifier,
        type_arguments: Vec<Box<TypeSpecifier>>,
        source_location: SourceLocation,
    },
    /// Type parameter (e.g., T in a generic function)
    TypeParameter {
        name: Identifier,
        constraints: Vec<TypeConstraint>,
        source_location: SourceLocation,
    },
    Array {
        element_type: Box<TypeSpecifier>,
        size: Option<Box<Expression>>,
        source_location: SourceLocation,
    },
    Map {
        key_type: Box<TypeSpecifier>,
        value_type: Box<TypeSpecifier>,
        source_location: SourceLocation,
    },
    Pointer {
        target_type: Box<TypeSpecifier>,
        is_mutable: bool,
        source_location: SourceLocation,
    },
    Function {
        parameter_types: Vec<Box<TypeSpecifier>>,
        return_type: Box<TypeSpecifier>,
        source_location: SourceLocation,
    },
    /// Ownership-annotated type
    Owned {
        base_type: Box<TypeSpecifier>,
        ownership: OwnershipKind,
        lifetime: Option<Identifier>,
        source_location: SourceLocation,
    },
}

/// Ownership kinds for type annotations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwnershipKind {
    /// Owned value (exclusive ownership) - ^T
    Owned,
    /// Borrowed reference (shared, immutable) - &T
    Borrowed,
    /// Mutable borrowed reference - &mut T
    BorrowedMut,
    /// Shared ownership (reference counted) - ~T
    Shared,
}

/// Primitive type names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    Integer,
    Integer32,
    Integer64,
    Float,
    Float32,
    Float64,
    String,
    Char,
    Boolean,
    Void,
    SizeT,
    UIntPtrT,
}

impl PrimitiveType {
    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            PrimitiveType::Integer
                | PrimitiveType::Integer32
                | PrimitiveType::Integer64
                | PrimitiveType::Float
                | PrimitiveType::Float32
                | PrimitiveType::Float64
                | PrimitiveType::SizeT
                | PrimitiveType::UIntPtrT
        )
    }
}

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Integer => write!(f, "INTEGER"),
            PrimitiveType::Integer32 => write!(f, "INTEGER32"),
            PrimitiveType::Integer64 => write!(f, "INTEGER64"),
            PrimitiveType::Float => write!(f, "FLOAT"),
            PrimitiveType::Float32 => write!(f, "FLOAT32"),
            PrimitiveType::Float64 => write!(f, "FLOAT64"),
            PrimitiveType::String => write!(f, "STRING"),
            PrimitiveType::Char => write!(f, "CHAR"),
            PrimitiveType::Boolean => write!(f, "BOOLEAN"),
            PrimitiveType::Void => write!(f, "VOID"),
            PrimitiveType::SizeT => write!(f, "SIZE_T"),
            PrimitiveType::UIntPtrT => write!(f, "UINTPTR_T"),
        }
    }
}

/// Constant declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDeclaration {
    pub name: Identifier,
    pub type_spec: Box<TypeSpecifier>,
    pub value: Box<Expression>,
    pub intent: Option<String>,
    pub source_location: SourceLocation,
}

/// Lifetime parameter for functions and types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeParameter {
    pub name: Identifier,
    pub source_location: SourceLocation,
}

/// Generic type parameter for functions and types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericParameter {
    pub name: Identifier,
    pub constraints: Vec<TypeConstraint>,
    pub default_type: Option<Box<TypeSpecifier>>,
    pub source_location: SourceLocation,
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: Identifier,
    pub intent: Option<String>,
    pub generic_parameters: Vec<GenericParameter>,
    pub lifetime_parameters: Vec<LifetimeParameter>,
    pub parameters: Vec<Parameter>,
    pub return_type: Box<TypeSpecifier>,
    pub metadata: FunctionMetadata,
    pub body: Block,
    pub export_info: Option<ExportInfo>,
    pub is_async: bool, // New async support
    pub source_location: SourceLocation,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: Identifier,
    pub param_type: Box<TypeSpecifier>,
    pub intent: Option<String>,
    pub constraint: Option<Box<Expression>>,
    pub passing_mode: PassingMode,
    pub source_location: SourceLocation,
}

/// Parameter passing modes for FFI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassingMode {
    ByValue,
    ByReference,
    ByPointer,
}

/// Function metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub preconditions: Vec<ContractAssertion>,
    pub postconditions: Vec<ContractAssertion>,
    pub invariants: Vec<ContractAssertion>,
    pub algorithm_hint: Option<String>,
    pub performance_expectation: Option<PerformanceExpectation>,
    pub complexity_expectation: Option<ComplexityExpectation>,
    pub throws_exceptions: Vec<Box<TypeSpecifier>>,
    pub thread_safe: Option<bool>,
    pub may_block: Option<bool>,
}

/// Contract assertion (precondition, postcondition, invariant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAssertion {
    pub condition: Box<Expression>,
    pub failure_action: FailureAction,
    pub message: Option<String>,
    pub source_location: SourceLocation,
}

/// Actions to take on contract failure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FailureAction {
    ThrowException,
    LogWarning,
    AssertFail,
}

/// Performance expectation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceExpectation {
    pub metric: PerformanceMetric,
    pub target_value: f64,
    pub context: Option<String>,
}

/// Performance metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceMetric {
    LatencyMs,
    ThroughputOpsPerSec,
    MemoryUsageBytes,
}

/// Complexity expectation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityExpectation {
    pub complexity_type: ComplexityType,
    pub notation: ComplexityNotation,
    pub value: String,
}

/// Types of complexity analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplexityType {
    Time,
    Space,
}

/// Complexity notation systems
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplexityNotation {
    BigO,
    BigTheta,
    BigOmega,
}

/// Export information for FFI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    pub export_type: ExportType,
    pub symbol_name: Option<String>,
    pub calling_convention: Option<CallingConvention>,
    pub package_name: Option<String>, // For Go
}

/// Export types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    CFunction,
    CStruct,
    RustFunction,
    RustStruct,
    GoFunction,
    GoStruct,
}

/// Calling conventions for FFI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallingConvention {
    C,
    StdCall,
    FastCall,
    System,
}

/// External function declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalFunction {
    pub name: Identifier,
    pub library: String, // Library name or "STATIC"
    pub symbol: Option<String>,
    pub parameters: Vec<Parameter>,
    pub return_type: Box<TypeSpecifier>,
    pub calling_convention: CallingConvention,
    pub thread_safe: bool,
    pub may_block: bool,
    pub variadic: bool,
    pub ownership_info: Option<OwnershipInfo>,
    pub source_location: SourceLocation,
}

/// Memory ownership information for FFI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipInfo {
    pub ownership: Ownership,
    pub lifetime: Option<Lifetime>,
    pub deallocator: Option<String>,
}

/// Memory ownership types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Ownership {
    CallerOwned,
    CalleeOwned,
    Borrowed,
    Shared,
}

/// Memory lifetime specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Lifetime {
    CallDuration,
    UntilNextCall,
    Static,
    Manual,
}

/// Block of statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub source_location: SourceLocation,
}

/// Statement types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    VariableDeclaration {
        name: Identifier,
        type_spec: Box<TypeSpecifier>,
        mutability: Mutability,
        initial_value: Option<Box<Expression>>,
        intent: Option<String>,
        source_location: SourceLocation,
    },
    Assignment {
        target: AssignmentTarget,
        value: Box<Expression>,
        source_location: SourceLocation,
    },
    FunctionCall {
        call: FunctionCall,
        source_location: SourceLocation,
    },
    Return {
        value: Option<Box<Expression>>,
        source_location: SourceLocation,
    },
    If {
        condition: Box<Expression>,
        then_block: Block,
        else_ifs: Vec<ElseIf>,
        else_block: Option<Block>,
        source_location: SourceLocation,
    },
    WhileLoop {
        condition: Box<Expression>,
        invariant: Option<String>,
        body: Block,
        label: Option<Identifier>,
        source_location: SourceLocation,
    },
    ForEachLoop {
        collection: Box<Expression>,
        element_binding: Identifier,
        element_type: Box<TypeSpecifier>,
        index_binding: Option<Identifier>,
        body: Block,
        label: Option<Identifier>,
        source_location: SourceLocation,
    },
    FixedIterationLoop {
        counter: Identifier,
        from_value: Box<Expression>,
        to_value: Box<Expression>,
        step_value: Option<Box<Expression>>,
        inclusive: bool,
        body: Block,
        label: Option<Identifier>,
        source_location: SourceLocation,
    },
    Break {
        target_label: Option<Identifier>,
        source_location: SourceLocation,
    },
    Continue {
        target_label: Option<Identifier>,
        source_location: SourceLocation,
    },
    TryBlock {
        protected_block: Block,
        catch_clauses: Vec<CatchClause>,
        finally_block: Option<Block>,
        source_location: SourceLocation,
    },
    Throw {
        exception: Box<Expression>,
        source_location: SourceLocation,
    },
    ResourceScope {
        scope: resource::ResourceScope,
        source_location: SourceLocation,
    },
    Expression {
        expr: Box<Expression>,
        source_location: SourceLocation,
    },
    Match {
        value: Box<Expression>,
        arms: Vec<MatchArm>,
        source_location: SourceLocation,
    },
    Concurrent {
        block: Block,
        source_location: SourceLocation,
    },
}

/// Variable mutability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mutability {
    Mutable,
    Immutable,
}

/// Assignment targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentTarget {
    Variable {
        name: Identifier,
    },
    ArrayElement {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    StructField {
        instance: Box<Expression>,
        field_name: Identifier,
    },
    MapValue {
        map: Box<Expression>,
        key: Box<Expression>,
    },
    Dereference {
        pointer: Box<Expression>,
    },
}

/// Else-if clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseIf {
    pub condition: Box<Expression>,
    pub block: Block,
    pub source_location: SourceLocation,
}

/// Exception catch clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchClause {
    pub exception_type: Box<TypeSpecifier>,
    pub binding_variable: Option<Identifier>,
    pub handler_block: Block,
    pub source_location: SourceLocation,
}

/// Expression types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    // Literals
    IntegerLiteral {
        value: i64,
        source_location: SourceLocation,
    },
    FloatLiteral {
        value: f64,
        source_location: SourceLocation,
    },
    StringLiteral {
        value: String,
        source_location: SourceLocation,
    },
    CharacterLiteral {
        value: char,
        source_location: SourceLocation,
    },
    BooleanLiteral {
        value: bool,
        source_location: SourceLocation,
    },
    NullLiteral {
        source_location: SourceLocation,
    },

    // Variables and identifiers
    Variable {
        name: Identifier,
        source_location: SourceLocation,
    },
    EnumMember {
        enum_type: Identifier,
        variant: Identifier,
        source_location: SourceLocation,
    },

    // Arithmetic expressions
    Add {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    Subtract {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    Multiply {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    Divide {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    IntegerDivide {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    Modulo {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    Negate {
        operand: Box<Expression>,
        source_location: SourceLocation,
    },

    // Comparison predicates
    Equals {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    NotEquals {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    LessThan {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    LessThanOrEqual {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    GreaterThan {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    GreaterThanOrEqual {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },

    // Logical expressions
    LogicalAnd {
        operands: Vec<Expression>,
        source_location: SourceLocation,
    },
    LogicalOr {
        operands: Vec<Expression>,
        source_location: SourceLocation,
    },
    LogicalNot {
        operand: Box<Expression>,
        source_location: SourceLocation,
    },

    // String operations
    StringConcat {
        operands: Vec<Expression>,
        source_location: SourceLocation,
    },
    StringLength {
        string: Box<Expression>,
        source_location: SourceLocation,
    },
    StringCharAt {
        string: Box<Expression>,
        index: Box<Expression>,
        source_location: SourceLocation,
    },
    Substring {
        string: Box<Expression>,
        start_index: Box<Expression>,
        length: Box<Expression>,
        source_location: SourceLocation,
    },
    StringEquals {
        left: Box<Expression>,
        right: Box<Expression>,
        source_location: SourceLocation,
    },
    StringContains {
        haystack: Box<Expression>,
        needle: Box<Expression>,
        source_location: SourceLocation,
    },

    // Type conversion
    TypeCast {
        value: Box<Expression>,
        target_type: Box<TypeSpecifier>,
        failure_behavior: CastFailureBehavior,
        source_location: SourceLocation,
    },

    // Function calls
    FunctionCall {
        call: FunctionCall,
        source_location: SourceLocation,
    },

    // Access operations
    FieldAccess {
        instance: Box<Expression>,
        field_name: Identifier,
        source_location: SourceLocation,
    },
    /// Method call: receiver.method(args)
    MethodCall {
        receiver: Box<Expression>,
        method_name: Identifier,
        arguments: Vec<Argument>,
        source_location: SourceLocation,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
        source_location: SourceLocation,
    },
    MapAccess {
        map: Box<Expression>,
        key: Box<Expression>,
        source_location: SourceLocation,
    },
    ArrayLength {
        array: Box<Expression>,
        source_location: SourceLocation,
    },

    // Pointer operations
    AddressOf {
        operand: Box<Expression>,
        mutability: bool,
        source_location: SourceLocation,
    },
    Dereference {
        pointer: Box<Expression>,
        source_location: SourceLocation,
    },
    PointerArithmetic {
        pointer: Box<Expression>,
        offset: Box<Expression>,
        operation: PointerOp,
        source_location: SourceLocation,
    },

    // Construction
    StructConstruct {
        type_name: Identifier,
        field_values: Vec<FieldValue>,
        source_location: SourceLocation,
    },
    ArrayLiteral {
        element_type: Box<TypeSpecifier>,
        elements: Vec<Box<Expression>>,
        source_location: SourceLocation,
    },
    MapLiteral {
        key_type: Box<TypeSpecifier>,
        value_type: Box<TypeSpecifier>,
        entries: Vec<MapEntry>,
        source_location: SourceLocation,
    },

    // Pattern matching
    Match {
        value: Box<Expression>,
        cases: Vec<MatchCase>,
        source_location: SourceLocation,
    },

    // Enum variant construction
    EnumVariant {
        enum_name: Identifier,
        variant_name: Identifier,
        values: Vec<Expression>,
        source_location: SourceLocation,
    },

    // Lambda/closure expression
    Lambda {
        /// Captured variables from enclosing scope
        captures: Vec<Capture>,
        parameters: Vec<Parameter>,
        return_type: Option<Box<TypeSpecifier>>,
        body: LambdaBody,
        source_location: SourceLocation,
    },

    /// Range expression: start..end or start..=end
    Range {
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
        inclusive: bool,
        source_location: SourceLocation,
    },
}

/// Lambda body - either a single expression or a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LambdaBody {
    Expression(Box<Expression>),
    Block(Block),
}

/// Capture mode for closure captures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureMode {
    /// Capture by value (copy)
    ByValue,
    /// Capture by reference (borrow)
    ByReference,
    /// Capture by mutable reference
    ByMutableReference,
}

/// A single capture in a closure capture list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capture {
    /// The name of the captured variable
    pub name: Identifier,
    /// How the variable is captured
    pub mode: CaptureMode,
    /// Source location
    pub source_location: SourceLocation,
}

/// Cast failure behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CastFailureBehavior {
    ThrowException,
    ReturnNullOrDefault,
}

/// Pointer arithmetic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PointerOp {
    Add,
    Subtract,
}

/// Function call representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub function_reference: FunctionReference,
    pub arguments: Vec<Argument>,
    pub variadic_arguments: Vec<Box<Expression>>, // For variadic functions
}

/// Function reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionReference {
    Local {
        name: Identifier,
    },
    Qualified {
        module: Identifier,
        name: Identifier,
    },
    External {
        name: Identifier,
    },
}

/// Function argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub parameter_name: Identifier,
    pub value: Box<Expression>,
    pub source_location: SourceLocation,
}

/// Struct field value in construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValue {
    pub field_name: Identifier,
    pub value: Box<Expression>,
    pub source_location: SourceLocation,
}

/// Map entry in map literal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEntry {
    pub key: Box<Expression>,
    pub value: Box<Expression>,
    pub source_location: SourceLocation,
}

/// Match case in pattern matching (for Expression::Match)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: Box<Expression>,
    pub source_location: SourceLocation,
}

/// Match arm in pattern matching (for Statement::Match with block bodies)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expression>>,
    pub body: Block,
    pub source_location: SourceLocation,
}

/// Pattern for pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    /// Match an enum variant (e.g., Ok value)
    EnumVariant {
        enum_name: Option<Identifier>, // None for unqualified variant
        variant_name: Identifier,
        bindings: Vec<Identifier>, // Variables to bind the associated values
        nested_pattern: Option<Box<Pattern>>, // For nested patterns like (Some (Ok x))
        source_location: SourceLocation,
    },
    /// Match a literal value
    Literal {
        value: Box<Expression>,
        source_location: SourceLocation,
    },
    /// Match a struct (e.g. Point { x: 0, y: 0 })
    Struct {
        struct_name: Identifier,
        fields: Vec<(Identifier, Pattern)>,
        source_location: SourceLocation,
    },
    /// Wildcard pattern (matches anything)
    Wildcard {
        binding: Option<Identifier>, // Variable to bind the matched value
        source_location: SourceLocation,
    },
}

/// Identifier representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub name: String,
    pub source_location: SourceLocation,
}

impl Identifier {
    pub fn new(name: String, source_location: SourceLocation) -> Self {
        Self {
            name,
            source_location,
        }
    }
}

/// Pretty printer for AST nodes
pub struct ASTPrettyPrinter {
    indent_level: usize,
    indent_size: usize,
}

impl ASTPrettyPrinter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            indent_size: 2,
        }
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn current_indent(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }

    pub fn print_program(&mut self, program: &Program) -> String {
        let mut result = String::new();
        result.push_str("Program {\n");
        self.indent();

        for module in &program.modules {
            result.push_str(&format!(
                "{}{}\n",
                self.current_indent(),
                self.print_module(module)
            ));
        }

        self.dedent();
        result.push_str("}\n");
        result
    }

    pub fn print_module(&mut self, module: &Module) -> String {
        let mut result = String::new();
        result.push_str(&format!("Module '{}' {{\n", module.name.name));
        self.indent();

        if let Some(intent) = &module.intent {
            result.push_str(&format!(
                "{}intent: \"{}\"\n",
                self.current_indent(),
                intent
            ));
        }

        for import in &module.imports {
            result.push_str(&format!(
                "{}{}\n",
                self.current_indent(),
                self.print_import(import)
            ));
        }

        for type_def in &module.type_definitions {
            result.push_str(&format!(
                "{}{}\n",
                self.current_indent(),
                self.print_type_definition(type_def)
            ));
        }

        for constant in &module.constant_declarations {
            result.push_str(&format!(
                "{}{}\n",
                self.current_indent(),
                self.print_constant_declaration(constant)
            ));
        }

        for function in &module.function_definitions {
            result.push_str(&format!(
                "{}{}\n",
                self.current_indent(),
                self.print_function(function)
            ));
        }

        self.dedent();
        result.push_str(&format!("{}}}", self.current_indent()));
        result
    }

    fn print_import(&self, import: &ImportStatement) -> String {
        if let Some(alias) = &import.alias {
            format!("import {} as {}", import.module_name.name, alias.name)
        } else {
            format!("import {}", import.module_name.name)
        }
    }

    fn print_type_definition(&self, type_def: &TypeDefinition) -> String {
        match type_def {
            TypeDefinition::Structured { name, fields, .. } => {
                let mut result = format!("struct {} {{\n", name.name);
                for field in fields {
                    result.push_str(&format!(
                        "  {}: {},\n",
                        field.name.name,
                        self.print_type_specifier(&field.field_type)
                    ));
                }
                result.push('}');
                result
            }
            TypeDefinition::Enumeration { name, variants, .. } => {
                let mut result = format!("enum {} {{\n", name.name);
                for variant in variants {
                    result.push_str(&format!("  {},\n", variant.name.name));
                }
                result.push('}');
                result
            }
            TypeDefinition::Alias {
                new_name,
                original_type,
                ..
            } => {
                format!(
                    "type {} = {}",
                    new_name.name,
                    self.print_type_specifier(original_type)
                )
            }
        }
    }

    fn print_type_specifier(&self, type_spec: &TypeSpecifier) -> String {
        match type_spec {
            TypeSpecifier::Primitive { type_name, .. } => format!("{:?}", type_name),
            TypeSpecifier::Named { name, .. } => name.name.clone(),
            TypeSpecifier::Array {
                element_type, size, ..
            } => {
                if let Some(size) = size {
                    format!(
                        "[{}; {}]",
                        self.print_type_specifier(element_type),
                        self.print_expression(size)
                    )
                } else {
                    format!("[{}]", self.print_type_specifier(element_type))
                }
            }
            TypeSpecifier::Map {
                key_type,
                value_type,
                ..
            } => {
                format!(
                    "Map<{}, {}>",
                    self.print_type_specifier(key_type),
                    self.print_type_specifier(value_type)
                )
            }
            TypeSpecifier::Pointer {
                target_type,
                is_mutable,
                ..
            } => {
                if *is_mutable {
                    format!("*mut {}", self.print_type_specifier(target_type))
                } else {
                    format!("*const {}", self.print_type_specifier(target_type))
                }
            }
            TypeSpecifier::Function {
                parameter_types,
                return_type,
                ..
            } => {
                let params: Vec<String> = parameter_types
                    .iter()
                    .map(|t| self.print_type_specifier(t))
                    .collect();
                format!(
                    "fn({}) -> {}",
                    params.join(", "),
                    self.print_type_specifier(return_type)
                )
            }
            TypeSpecifier::Generic {
                base_type,
                type_arguments,
                ..
            } => {
                let args: Vec<String> = type_arguments
                    .iter()
                    .map(|t| self.print_type_specifier(t))
                    .collect();
                format!("{}<{}>", base_type.name, args.join(", "))
            }
            TypeSpecifier::Owned {
                ownership,
                base_type,
                lifetime,
                ..
            } => {
                let prefix = match ownership {
                    OwnershipKind::Owned => "^",
                    OwnershipKind::Borrowed => "&",
                    OwnershipKind::BorrowedMut => "&mut ",
                    OwnershipKind::Shared => "~",
                };
                let lifetime_str = if let Some(lt) = lifetime {
                    format!("'{}", lt.name)
                } else {
                    "".to_string()
                };
                format!("{}{}{}", prefix, lifetime_str, self.print_type_specifier(base_type))
            }
            TypeSpecifier::TypeParameter {
                name, constraints, ..
            } => {
                if constraints.is_empty() {
                    name.name.clone()
                } else {
                    // For now, just show the parameter name without constraints
                    // Full constraint printing would be more complex
                    format!("{}: <constraints>", name.name)
                }
            }
        }
    }

    fn print_constant_declaration(&self, constant: &ConstantDeclaration) -> String {
        format!(
            "const {}: {} = {}",
            constant.name.name,
            self.print_type_specifier(&constant.type_spec),
            self.print_expression(&constant.value)
        )
    }

    fn print_function(&mut self, function: &Function) -> String {
        let mut result = format!("fn {}(", function.name.name);

        let params: Vec<String> = function
            .parameters
            .iter()
            .map(|p| {
                format!(
                    "{}: {}",
                    p.name.name,
                    self.print_type_specifier(&p.param_type)
                )
            })
            .collect();
        result.push_str(&params.join(", "));
        result.push_str(&format!(
            ") -> {} {{\n",
            self.print_type_specifier(&function.return_type)
        ));

        self.indent();
        result.push_str(&self.print_block(&function.body));
        self.dedent();
        result.push_str(&format!("{}}}", self.current_indent()));
        result
    }

    fn print_block(&mut self, block: &Block) -> String {
        let mut result = String::new();
        for statement in &block.statements {
            result.push_str(&format!(
                "{}{}\n",
                self.current_indent(),
                self.print_statement(statement)
            ));
        }
        result
    }

    fn print_statement(&mut self, statement: &Statement) -> String {
        match statement {
            Statement::VariableDeclaration {
                name,
                type_spec,
                mutability,
                initial_value,
                ..
            } => {
                let mut result = format!(
                    "{} {}: {}",
                    match mutability {
                        Mutability::Mutable => "let mut",
                        Mutability::Immutable => "let",
                    },
                    name.name,
                    self.print_type_specifier(type_spec)
                );
                if let Some(value) = initial_value {
                    result.push_str(&format!(" = {}", self.print_expression(value)));
                }
                result.push(';');
                result
            }
            Statement::Assignment { target, value, .. } => {
                format!(
                    "{} = {};",
                    self.print_assignment_target(target),
                    self.print_expression(value)
                )
            }
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    format!("return {};", self.print_expression(value))
                } else {
                    "return;".to_string()
                }
            }
            Statement::Concurrent { block, .. } => {
                let mut result = "concurrent {\n".to_string();
                self.indent();
                result.push_str(&self.print_block(block));
                self.dedent();
                result.push_str(&format!("{}}}", self.current_indent()));
                result
            }
            _ => "/* other statement */".to_string(),
        }
    }

    fn print_assignment_target(&self, target: &AssignmentTarget) -> String {
        match target {
            AssignmentTarget::Variable { name } => name.name.clone(),
            AssignmentTarget::ArrayElement { array, index } => {
                format!(
                    "{}[{}]",
                    self.print_expression(array),
                    self.print_expression(index)
                )
            }
            AssignmentTarget::StructField {
                instance,
                field_name,
            } => {
                format!("{}.{}", self.print_expression(instance), field_name.name)
            }
            AssignmentTarget::MapValue { map, key } => {
                format!(
                    "{}[{}]",
                    self.print_expression(map),
                    self.print_expression(key)
                )
            }
            AssignmentTarget::Dereference { pointer } => {
                format!("*{}", self.print_expression(pointer))
            }
        }
    }

    fn print_expression(&self, expression: &Expression) -> String {
        match expression {
            Expression::IntegerLiteral { value, .. } => value.to_string(),
            Expression::FloatLiteral { value, .. } => value.to_string(),
            Expression::StringLiteral { value, .. } => format!("\"{}\"", value),
            Expression::BooleanLiteral { value, .. } => value.to_string(),
            Expression::NullLiteral { .. } => "null".to_string(),
            Expression::Variable { name, .. } => name.name.clone(),
            Expression::Add { left, right, .. } => {
                format!(
                    "({} + {})",
                    self.print_expression(left),
                    self.print_expression(right)
                )
            }
            Expression::FunctionCall { call, .. } => {
                let func_name = match &call.function_reference {
                    FunctionReference::Local { name } => name.name.clone(),
                    FunctionReference::Qualified { module, name } => {
                        format!("{}.{}", module.name, name.name)
                    }
                    FunctionReference::External { name } => name.name.clone(),
                };
                let args: Vec<String> = call
                    .arguments
                    .iter()
                    .map(|arg| {
                        format!(
                            "{}: {}",
                            arg.parameter_name.name,
                            self.print_expression(&arg.value)
                        )
                    })
                    .collect();
                format!("{}({})", func_name, args.join(", "))
            }
            _ => "/* expression */".to_string(),
        }
    }
}

impl Default for ASTPrettyPrinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_creation() {
        let loc = SourceLocation::new("test.aether".to_string(), 1, 1, 0);
        let id = Identifier::new("test_identifier".to_string(), loc.clone());

        assert_eq!(id.name, "test_identifier");
        assert_eq!(id.source_location, loc);
    }

    #[test]
    fn test_ast_pretty_printer() {
        let mut printer = ASTPrettyPrinter::new();
        let loc = SourceLocation::new("test.aether".to_string(), 1, 1, 0);

        let module = Module {
            name: Identifier::new("test_module".to_string(), loc.clone()),
            intent: Some("Test module".to_string()),
            imports: vec![],
            exports: vec![],
            type_definitions: vec![],
            constant_declarations: vec![],
            function_definitions: vec![],
            external_functions: vec![],
            source_location: loc,
        };

        let output = printer.print_module(&module);
        assert!(output.contains("Module 'test_module'"));
        assert!(output.contains("intent: \"Test module\""));
    }

    #[test]
    fn test_expression_serialization() {
        let loc = SourceLocation::new("test.aether".to_string(), 1, 1, 0);
        let expr = Expression::IntegerLiteral {
            value: 42,
            source_location: loc,
        };

        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            Expression::IntegerLiteral { value, .. } => assert_eq!(value, 42),
            _ => panic!("Deserialization failed"),
        }
    }
}

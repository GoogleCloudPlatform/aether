//! Middle Intermediate Representation (MIR) for AetherScript
//! 
//! MIR is a lower-level representation that uses SSA form and basic blocks
//! for optimization and analysis. It serves as the bridge between the AST
//! and the final code generation phase.

pub mod lowering;
pub mod dataflow;
pub mod validation;

use crate::types::Type;
use crate::error::SourceLocation;
use std::collections::HashMap;
use std::fmt;

/// A MIR program consists of multiple functions
#[derive(Debug, Clone)]
pub struct Program {
    pub functions: HashMap<String, Function>,
    pub global_constants: HashMap<String, Constant>,
    pub external_functions: HashMap<String, ExternalFunction>,
    pub type_definitions: HashMap<String, crate::types::TypeDefinition>,
}

/// A MIR function in SSA form
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub locals: HashMap<LocalId, Local>,
    pub basic_blocks: HashMap<BasicBlockId, BasicBlock>,
    pub entry_block: BasicBlockId,
    pub return_local: Option<LocalId>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub local_id: LocalId,
}

/// Local variable/temporary in SSA form
#[derive(Debug, Clone)]
pub struct Local {
    pub ty: Type,
    pub is_mutable: bool,
    pub source_info: Option<SourceInfo>,
}

/// Source information for debugging
#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub span: SourceLocation,
    pub scope: ScopeId,
}

/// A basic block contains a sequence of statements and a terminator
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

/// MIR statements (non-branching operations)
#[derive(Debug, Clone)]
pub enum Statement {
    /// Assignment: local = rvalue
    Assign {
        place: Place,
        rvalue: Rvalue,
        source_info: SourceInfo,
    },
    
    /// Storage marker for lifetime analysis
    StorageLive(LocalId),
    StorageDead(LocalId),
    
    /// No-op (used for placeholders)
    Nop,
}

/// Right-hand side of assignments
#[derive(Debug, Clone)]
pub enum Rvalue {
    /// Use of an operand
    Use(Operand),
    
    /// Binary operation
    BinaryOp {
        op: BinOp,
        left: Operand,
        right: Operand,
    },
    
    /// Unary operation
    UnaryOp {
        op: UnOp,
        operand: Operand,
    },
    
    /// Function call
    Call {
        func: Operand,
        args: Vec<Operand>,
    },
    
    /// Aggregate construction (arrays, structs)
    Aggregate {
        kind: AggregateKind,
        operands: Vec<Operand>,
    },
    
    /// Cast operation
    Cast {
        kind: CastKind,
        operand: Operand,
        ty: Type,
    },
    
    /// Reference/address-of operation
    Ref {
        place: Place,
        mutability: Mutability,
    },
    
    /// Array/slice length
    Len(Place),
    
    /// Discriminant for enums
    Discriminant(Place),
}

/// Operands (values that can be used)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    /// Copy a place
    Copy(Place),
    
    /// Move a place (consumes it)
    Move(Place),
    
    /// Constant value
    Constant(Constant),
}

/// Places (lvalues that can be assigned to)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Place {
    pub local: LocalId,
    pub projection: Vec<PlaceElem>,
}

/// Place projections (field access, array indexing, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlaceElem {
    /// Dereference
    Deref,
    
    /// Field access
    Field {
        field: FieldIdx,
        ty: Type,
    },
    
    /// Array/slice indexing
    Index(LocalId),
    
    /// Subslice
    Subslice {
        from: u64,
        to: Option<u64>,
    },
}

/// Block terminators (control flow)
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Unconditional jump
    Goto {
        target: BasicBlockId,
    },
    
    /// Conditional branch
    SwitchInt {
        discriminant: Operand,
        switch_ty: Type,
        targets: SwitchTargets,
    },
    
    /// Function return
    Return,
    
    /// Unreachable code
    Unreachable,
    
    /// Function call with landing pad
    Call {
        func: Operand,
        args: Vec<Operand>,
        destination: Place,
        target: Option<BasicBlockId>,
        cleanup: Option<BasicBlockId>,
    },
    
    /// Drop (destructor call)
    Drop {
        place: Place,
        target: BasicBlockId,
        unwind: Option<BasicBlockId>,
    },
    
    /// Assert (runtime check)
    Assert {
        condition: Operand,
        expected: bool,
        message: AssertMessage,
        target: BasicBlockId,
        cleanup: Option<BasicBlockId>,
    },
}

/// Switch targets for conditional branches
#[derive(Debug, Clone)]
pub struct SwitchTargets {
    pub values: Vec<u128>,
    pub targets: Vec<BasicBlockId>,
    pub otherwise: BasicBlockId,
}

/// Assertion messages
#[derive(Debug, Clone)]
pub enum AssertMessage {
    BoundsCheck { len: Operand, index: Operand },
    Overflow(BinOp, Operand, Operand),
    DivisionByZero(Operand),
    RemainderByZero(Operand),
    Custom(String),
}

/// Binary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Mod,
    
    // Bitwise
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    
    // Logical
    And,
    Or,
    
    // Pointer operations
    Offset,
}

/// Unary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnOp {
    Not,
    Neg,
}

/// Cast kinds
#[derive(Debug, Clone, Copy)]
pub enum CastKind {
    /// Numeric cast (int to float, etc.)
    Numeric,
    
    /// Pointer to pointer cast
    Pointer,
    
    /// Unsizing cast (e.g., array to slice)
    Unsize,
}

/// Aggregate kinds
#[derive(Debug, Clone)]
pub enum AggregateKind {
    Array(Type),
    Tuple,
    Struct(String, Vec<String>), // struct name and field names
    Enum(String, String),         // enum name and variant name
}

/// Mutability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Not,
    Mut,
}

/// Constant values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constant {
    pub ty: Type,
    pub value: ConstantValue,
}

/// Constant value representations
#[derive(Debug, Clone)]
pub enum ConstantValue {
    Bool(bool),
    Integer(i128),
    Float(f64),
    String(String),
    Char(char),
    Null,
}

impl PartialEq for ConstantValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ConstantValue::Bool(a), ConstantValue::Bool(b)) => a == b,
            (ConstantValue::Integer(a), ConstantValue::Integer(b)) => a == b,
            (ConstantValue::Float(a), ConstantValue::Float(b)) => (a - b).abs() < f64::EPSILON,
            (ConstantValue::String(a), ConstantValue::String(b)) => a == b,
            (ConstantValue::Char(a), ConstantValue::Char(b)) => a == b,
            (ConstantValue::Null, ConstantValue::Null) => true,
            _ => false,
        }
    }
}

impl Eq for ConstantValue {}

impl std::hash::Hash for ConstantValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ConstantValue::Bool(b) => {
                0u8.hash(state);
                b.hash(state);
            }
            ConstantValue::Integer(i) => {
                1u8.hash(state);
                i.hash(state);
            }
            ConstantValue::Float(f) => {
                2u8.hash(state);
                // Hash the bit representation of the float
                f.to_bits().hash(state);
            }
            ConstantValue::String(s) => {
                3u8.hash(state);
                s.hash(state);
            }
            ConstantValue::Char(c) => {
                4u8.hash(state);
                c.hash(state);
            }
            ConstantValue::Null => {
                5u8.hash(state);
            }
        }
    }
}

/// External function declaration
#[derive(Debug, Clone)]
pub struct ExternalFunction {
    pub name: String,
    pub parameters: Vec<Type>,
    pub return_type: Type,
    pub calling_convention: CallingConvention,
    pub variadic: bool,
}

/// Calling conventions
#[derive(Debug, Clone, Copy)]
pub enum CallingConvention {
    Rust,
    C,
    System,
}

/// Type definitions for IDs
pub type LocalId = u32;
pub type BasicBlockId = u32;
pub type FieldIdx = u32;
pub type ScopeId = u32;

/// MIR builder for constructing MIR from AST
pub struct Builder {
    /// Current function being built
    current_function: Option<Function>,
    
    /// Next local ID
    next_local_id: LocalId,
    
    /// Next basic block ID
    next_block_id: BasicBlockId,
    
    /// Current basic block
    pub current_block: Option<BasicBlockId>,
    
    /// Scope stack
    scopes: Vec<Scope>,
    
    /// Next scope ID
    next_scope_id: ScopeId,
}

/// Scope information for the builder
struct Scope {
    parent: Option<ScopeId>,
    variables: HashMap<String, LocalId>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self {
            current_function: None,
            next_local_id: 0,
            next_block_id: 0,
            current_block: None,
            scopes: Vec::new(),
            next_scope_id: 0,
        }
    }
    
    /// Start building a new function
    pub fn start_function(&mut self, name: String, params: Vec<(String, Type)>, return_type: Type) {
        let function = Function {
            name: name.clone(),
            parameters: Vec::new(),
            return_type,
            locals: HashMap::new(),
            basic_blocks: HashMap::new(),
            entry_block: 0,
            return_local: None,
        };
        
        self.current_function = Some(function);
        
        // Create locals for parameters
        for (param_name, param_type) in params {
            let local_id = self.new_local(param_type.clone(), false);
            if let Some(func) = &mut self.current_function {
                func.parameters.push(Parameter {
                    name: param_name,
                    ty: param_type,
                    local_id,
                });
            }
        }
        
        // Create entry block
        let entry_block = self.new_block();
        if let Some(func) = &mut self.current_function {
            func.entry_block = entry_block;
        }
        self.current_block = Some(entry_block);
    }
    
    /// Finish building the current function
    pub fn finish_function(&mut self) -> Function {
        self.current_function.take().expect("No function being built")
    }
    
    /// Create a new local
    pub fn new_local(&mut self, ty: Type, is_mutable: bool) -> LocalId {
        let local_id = self.next_local_id;
        self.next_local_id += 1;
        
        if let Some(func) = &mut self.current_function {
            func.locals.insert(local_id, Local {
                ty,
                is_mutable,
                source_info: None,
            });
        }
        
        local_id
    }
    
    /// Create a new basic block
    pub fn new_block(&mut self) -> BasicBlockId {
        let block_id = self.next_block_id;
        self.next_block_id += 1;
        
        if let Some(func) = &mut self.current_function {
            func.basic_blocks.insert(block_id, BasicBlock {
                id: block_id,
                statements: Vec::new(),
                terminator: Terminator::Unreachable,
            });
        }
        
        block_id
    }
    
    /// Switch to a different basic block
    pub fn switch_to_block(&mut self, block_id: BasicBlockId) {
        self.current_block = Some(block_id);
    }
    
    /// Add a statement to the current block
    pub fn push_statement(&mut self, statement: Statement) {
        if let (Some(func), Some(block_id)) = (&mut self.current_function, self.current_block) {
            if let Some(block) = func.basic_blocks.get_mut(&block_id) {
                block.statements.push(statement);
            }
        }
    }
    
    /// Set the terminator for the current block
    pub fn set_terminator(&mut self, terminator: Terminator) {
        if let (Some(func), Some(block_id)) = (&mut self.current_function, self.current_block) {
            if let Some(block) = func.basic_blocks.get_mut(&block_id) {
                block.terminator = terminator;
            }
        }
    }
    
    /// Push a new scope
    pub fn push_scope(&mut self) -> ScopeId {
        let scope_id = self.next_scope_id;
        self.next_scope_id += 1;
        
        self.scopes.push(Scope {
            parent: Some(scope_id),
            variables: HashMap::new(),
        });
        
        scope_id
    }
    
    /// Pop a scope and emit storage dead statements
    pub fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            // Emit StorageDead for all locals in this scope
            for local in scope.variables.values() {
                self.push_statement(Statement::StorageDead(*local));
            }
        }
    }
}

/// Control Flow Graph (CFG) utilities
pub mod cfg {
    use super::*;
    
    /// Get predecessors of a basic block
    pub fn predecessors(func: &Function, block_id: BasicBlockId) -> Vec<BasicBlockId> {
        let mut preds = Vec::new();
        
        for (pred_id, block) in &func.basic_blocks {
            match &block.terminator {
                Terminator::Goto { target } => {
                    if *target == block_id {
                        preds.push(*pred_id);
                    }
                }
                Terminator::SwitchInt { targets, .. } => {
                    if targets.targets.contains(&block_id) || targets.otherwise == block_id {
                        preds.push(*pred_id);
                    }
                }
                Terminator::Call { target, cleanup, .. } => {
                    if target.as_ref() == Some(&block_id) || cleanup.as_ref() == Some(&block_id) {
                        preds.push(*pred_id);
                    }
                }
                Terminator::Drop { target, unwind, .. } => {
                    if *target == block_id || unwind.as_ref() == Some(&block_id) {
                        preds.push(*pred_id);
                    }
                }
                Terminator::Assert { target, cleanup, .. } => {
                    if *target == block_id || cleanup.as_ref() == Some(&block_id) {
                        preds.push(*pred_id);
                    }
                }
                _ => {}
            }
        }
        
        preds
    }
    
    /// Get successors of a basic block
    pub fn successors(block: &BasicBlock) -> Vec<BasicBlockId> {
        match &block.terminator {
            Terminator::Goto { target } => vec![*target],
            Terminator::SwitchInt { targets, .. } => {
                let mut succs = targets.targets.clone();
                succs.push(targets.otherwise);
                succs
            }
            Terminator::Return | Terminator::Unreachable => vec![],
            Terminator::Call { target, cleanup, .. } => {
                let mut succs = Vec::new();
                if let Some(t) = target {
                    succs.push(*t);
                }
                if let Some(c) = cleanup {
                    succs.push(*c);
                }
                succs
            }
            Terminator::Drop { target, unwind, .. } => {
                let mut succs = vec![*target];
                if let Some(u) = unwind {
                    succs.push(*u);
                }
                succs
            }
            Terminator::Assert { target, cleanup, .. } => {
                let mut succs = vec![*target];
                if let Some(c) = cleanup {
                    succs.push(*c);
                }
                succs
            }
        }
    }
}

/// Pretty printer for MIR
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fn {}({}) -> {} {{", 
            self.name,
            self.parameters.iter()
                .map(|p| format!("{}: {}", p.name, p.ty))
                .collect::<Vec<_>>()
                .join(", "),
            self.return_type
        )?;
        
        // Print locals
        for (id, local) in &self.locals {
            writeln!(f, "    let _{}: {};", id, local.ty)?;
        }
        
        writeln!(f)?;
        
        // Print basic blocks
        for block_id in 0..self.basic_blocks.len() as u32 {
            if let Some(block) = self.basic_blocks.get(&block_id) {
                writeln!(f, "  bb{}:", block_id)?;
                
                for stmt in &block.statements {
                    writeln!(f, "    {:?}", stmt)?;
                }
                
                writeln!(f, "    {:?}", block.terminator)?;
                writeln!(f)?;
            }
        }
        
        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;
    use crate::ast::PrimitiveType;
    
    #[test]
    fn test_mir_builder_basic() {
        let mut builder = Builder::new();
        
        // Build a simple function: fn add(x: i32, y: i32) -> i32
        builder.start_function(
            "add".to_string(),
            vec![
                ("x".to_string(), Type::primitive(PrimitiveType::Integer)),
                ("y".to_string(), Type::primitive(PrimitiveType::Integer)),
            ],
            Type::primitive(PrimitiveType::Integer),
        );
        
        // Create a temporary for the result
        let result = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        
        // Add statement: _2 = _0 + _1
        builder.push_statement(Statement::Assign {
            place: Place {
                local: result,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Add,
                left: Operand::Copy(Place { local: 0, projection: vec![] }),
                right: Operand::Copy(Place { local: 1, projection: vec![] }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });
        
        // Set terminator: return _2
        builder.set_terminator(Terminator::Return);
        
        let function = builder.finish_function();
        
        assert_eq!(function.name, "add");
        assert_eq!(function.parameters.len(), 2);
        assert_eq!(function.locals.len(), 3); // 2 params + 1 temp
        assert_eq!(function.basic_blocks.len(), 1);
    }
    
    #[test]
    fn test_cfg_analysis() {
        let mut builder = Builder::new();
        
        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Void),
        );
        
        let bb0 = builder.current_block.unwrap();
        let bb1 = builder.new_block();
        let bb2 = builder.new_block();
        
        // bb0 -> bb1, bb2
        builder.switch_to_block(bb0);
        builder.set_terminator(Terminator::SwitchInt {
            discriminant: Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Boolean),
                value: ConstantValue::Bool(true),
            }),
            switch_ty: Type::primitive(PrimitiveType::Boolean),
            targets: SwitchTargets {
                values: vec![1],
                targets: vec![bb1],
                otherwise: bb2,
            },
        });
        
        // bb1 -> bb2
        builder.switch_to_block(bb1);
        builder.set_terminator(Terminator::Goto { target: bb2 });
        
        // bb2 -> return
        builder.switch_to_block(bb2);
        builder.set_terminator(Terminator::Return);
        
        let function = builder.finish_function();
        
        // Test predecessors
        let bb2_preds = cfg::predecessors(&function, bb2);
        assert_eq!(bb2_preds.len(), 2);
        assert!(bb2_preds.contains(&bb0));
        assert!(bb2_preds.contains(&bb1));
        
        // Test successors
        let bb0_succs = cfg::successors(&function.basic_blocks[&bb0]);
        assert_eq!(bb0_succs.len(), 2);
        assert!(bb0_succs.contains(&bb1));
        assert!(bb0_succs.contains(&bb2));
    }
    
    #[test]
    fn test_constant_values() {
        let bool_const = Constant {
            ty: Type::primitive(PrimitiveType::Boolean),
            value: ConstantValue::Bool(true),
        };
        
        let int_const = Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(42),
        };
        
        let float_const = Constant {
            ty: Type::primitive(PrimitiveType::Float),
            value: ConstantValue::Float(3.14),
        };
        
        let string_const = Constant {
            ty: Type::primitive(PrimitiveType::String),
            value: ConstantValue::String("hello".to_string()),
        };
        
        match bool_const.value {
            ConstantValue::Bool(v) => assert!(v),
            _ => panic!("Wrong constant type"),
        }
        
        match int_const.value {
            ConstantValue::Integer(v) => assert_eq!(v, 42),
            _ => panic!("Wrong constant type"),
        }
        
        match float_const.value {
            ConstantValue::Float(v) => assert!((v - 3.14).abs() < f64::EPSILON),
            _ => panic!("Wrong constant type"),
        }
        
        match string_const.value {
            ConstantValue::String(ref v) => assert_eq!(v, "hello"),
            _ => panic!("Wrong constant type"),
        }
    }
}
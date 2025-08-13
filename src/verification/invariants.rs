//! Loop invariants and global invariants for verification

use crate::error::SourceLocation;
use crate::mir::BasicBlockId;
use super::contracts::Expression;

/// Loop invariant specification
#[derive(Debug, Clone)]
pub struct LoopInvariant {
    /// The loop header block
    pub loop_header: BasicBlockId,
    
    /// Invariant conditions
    pub conditions: Vec<InvariantCondition>,
    
    /// Loop variant (for termination)
    pub variant: Option<LoopVariant>,
}

/// A single invariant condition
#[derive(Debug, Clone)]
pub struct InvariantCondition {
    /// Condition name/label
    pub name: String,
    
    /// The invariant expression
    pub expression: Expression,
    
    /// Source location
    pub location: SourceLocation,
}

/// Loop variant for proving termination
#[derive(Debug, Clone)]
pub struct LoopVariant {
    /// The variant expression (must decrease)
    pub expression: Expression,
    
    /// Lower bound (usually 0)
    pub lower_bound: Expression,
}

/// Global invariant that must hold throughout execution
#[derive(Debug, Clone)]
pub struct GlobalInvariant {
    /// Invariant name
    pub name: String,
    
    /// The invariant expression
    pub expression: Expression,
    
    /// When this invariant is active
    pub scope: InvariantScope,
    
    /// Source location
    pub location: SourceLocation,
}

/// Scope where an invariant applies
#[derive(Debug, Clone)]
pub enum InvariantScope {
    /// Always active
    Always,
    
    /// Active within a specific function
    Function(String),
    
    /// Active within a module
    Module(String),
    
    /// Active when a condition holds
    Conditional(Expression),
}

impl LoopInvariant {
    /// Create a new loop invariant
    pub fn new(loop_header: BasicBlockId) -> Self {
        Self {
            loop_header,
            conditions: Vec::new(),
            variant: None,
        }
    }
    
    /// Add an invariant condition
    pub fn add_condition(&mut self, name: String, expr: Expression, location: SourceLocation) {
        self.conditions.push(InvariantCondition {
            name,
            expression: expr,
            location,
        });
    }
    
    /// Set the loop variant
    pub fn set_variant(&mut self, expr: Expression, lower_bound: Expression) {
        self.variant = Some(LoopVariant {
            expression: expr,
            lower_bound,
        });
    }
    
    /// Check if this invariant has a variant (for termination checking)
    pub fn has_variant(&self) -> bool {
        self.variant.is_some()
    }
}

impl GlobalInvariant {
    /// Create a new global invariant
    pub fn new(name: String, expr: Expression, scope: InvariantScope, location: SourceLocation) -> Self {
        Self {
            name,
            expression: expr,
            scope,
            location,
        }
    }
    
    /// Check if this invariant applies in a given context
    pub fn applies_in_function(&self, function_name: &str) -> bool {
        match &self.scope {
            InvariantScope::Always => true,
            InvariantScope::Function(f) => f == function_name,
            InvariantScope::Module(_) => true, // Simplified for now
            InvariantScope::Conditional(_) => true, // Need runtime check
        }
    }
}

/// Extract loop invariants from MIR
pub fn extract_loop_invariants(_function: &crate::mir::Function) -> Vec<LoopInvariant> {
    // In a real implementation, we would:
    // 1. Identify loop headers in the CFG
    // 2. Look for invariant annotations in the source
    // 3. Possibly infer simple invariants automatically
    
    // For now, return empty vec
    Vec::new()
}

/// Common invariant patterns
pub mod patterns {
    use super::*;
    use crate::verification::contracts::{BinaryOp, ConstantValue};
    
    /// Create an array bounds invariant: 0 <= index < length
    pub fn array_bounds(index_var: &str, length_expr: Expression) -> Expression {
        Expression::BinaryOp {
            op: BinaryOp::And,
            left: Box::new(Expression::BinaryOp {
                op: BinaryOp::Le,
                left: Box::new(Expression::Constant(ConstantValue::Integer(0))),
                right: Box::new(Expression::Variable(index_var.to_string())),
            }),
            right: Box::new(Expression::BinaryOp {
                op: BinaryOp::Lt,
                left: Box::new(Expression::Variable(index_var.to_string())),
                right: Box::new(length_expr),
            }),
        }
    }
    
    /// Create a non-null invariant
    pub fn non_null(var: &str) -> Expression {
        Expression::BinaryOp {
            op: BinaryOp::Ne,
            left: Box::new(Expression::Variable(var.to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Null)),
        }
    }
    
    /// Create a range invariant: low <= var <= high
    pub fn in_range(var: &str, low: i64, high: i64) -> Expression {
        Expression::BinaryOp {
            op: BinaryOp::And,
            left: Box::new(Expression::BinaryOp {
                op: BinaryOp::Le,
                left: Box::new(Expression::Constant(ConstantValue::Integer(low))),
                right: Box::new(Expression::Variable(var.to_string())),
            }),
            right: Box::new(Expression::BinaryOp {
                op: BinaryOp::Le,
                left: Box::new(Expression::Variable(var.to_string())),
                right: Box::new(Expression::Constant(ConstantValue::Integer(high))),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::patterns;
    
    #[test]
    fn test_loop_invariant() {
        let mut inv = LoopInvariant::new(0);
        
        // Add invariant: i >= 0
        inv.add_condition(
            "non_negative".to_string(),
            Expression::BinaryOp {
                op: crate::verification::contracts::BinaryOp::Ge,
                left: Box::new(Expression::Variable("i".to_string())),
                right: Box::new(Expression::Constant(
                    crate::verification::contracts::ConstantValue::Integer(0)
                )),
            },
            SourceLocation::unknown(),
        );
        
        assert_eq!(inv.conditions.len(), 1);
        assert!(!inv.has_variant());
        
        // Add variant: n - i
        inv.set_variant(
            Expression::BinaryOp {
                op: crate::verification::contracts::BinaryOp::Sub,
                left: Box::new(Expression::Variable("n".to_string())),
                right: Box::new(Expression::Variable("i".to_string())),
            },
            Expression::Constant(crate::verification::contracts::ConstantValue::Integer(0)),
        );
        
        assert!(inv.has_variant());
    }
    
    #[test]
    fn test_array_bounds_pattern() {
        let bounds = patterns::array_bounds(
            "i",
            Expression::Length(Box::new(Expression::Variable("arr".to_string()))),
        );
        
        assert_eq!(bounds.to_string(), "((0 <= i) && (i < len(arr)))");
    }
}
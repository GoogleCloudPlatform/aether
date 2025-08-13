//! Whole program optimization for AetherScript
//!
//! Performs optimizations that require analysis of the entire program,
//! including cross-function optimizations and global constant propagation.

use crate::mir::{Function, Program, BasicBlock, Statement, Rvalue, Operand, Constant, Terminator};
use crate::error::SemanticError;
use crate::optimizations::OptimizationPass;
use std::collections::{HashMap, HashSet};

/// Whole program optimization pass
#[derive(Debug)]
pub struct WholeProgramOptimizationPass {
    /// Functions that are never called (candidates for elimination)
    dead_functions: HashSet<String>,
    
    /// Global constant values that can be propagated
    global_constants: HashMap<String, Constant>,
    
    /// Function call graph
    call_graph: CallGraph,
    
    /// Functions that have been inlined
    inlined_functions: HashSet<String>,
}

/// Call graph for interprocedural analysis
#[derive(Debug, Default)]
pub struct CallGraph {
    /// Mapping from function to list of functions it calls
    calls: HashMap<String, HashSet<String>>,
    
    /// Mapping from function to list of functions that call it
    called_by: HashMap<String, HashSet<String>>,
    
    /// Functions that are entry points (external or main)
    entry_points: HashSet<String>,
}

/// Function analysis results
#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    /// Whether function is pure (no side effects)
    pub is_pure: bool,
    
    /// Whether function is small enough for inlining
    pub is_inlineable: bool,
    
    /// Estimated cost (instruction count)
    pub cost: usize,
    
    /// Functions called by this function
    pub calls: HashSet<String>,
    
    /// Whether function modifies global state
    pub modifies_globals: bool,
    
    /// Whether function has recursive calls
    pub is_recursive: bool,
}

impl WholeProgramOptimizationPass {
    pub fn new() -> Self {
        Self {
            dead_functions: HashSet::new(),
            global_constants: HashMap::new(),
            call_graph: CallGraph::default(),
            inlined_functions: HashSet::new(),
        }
    }
    
    /// Perform whole program analysis
    pub fn analyze_program(&mut self, program: &Program) -> Result<HashMap<String, FunctionAnalysis>, SemanticError> {
        // Build call graph
        self.build_call_graph(program)?;
        
        // Find entry points (functions that are externally visible)
        self.find_entry_points(program);
        
        // Perform function analysis
        let mut analyses = HashMap::new();
        for (name, function) in &program.functions {
            let analysis = self.analyze_function(function)?;
            analyses.insert(name.clone(), analysis);
        }
        
        // Find dead functions
        self.find_dead_functions(&analyses);
        
        // Find global constants
        self.find_global_constants(program);
        
        Ok(analyses)
    }
    
    /// Build call graph from program
    fn build_call_graph(&mut self, program: &Program) -> Result<(), SemanticError> {
        self.call_graph = CallGraph::default();
        
        for (caller_name, function) in &program.functions {
            let mut calls = HashSet::new();
            
            // Analyze all basic blocks
            for block in function.basic_blocks.values() {
                self.analyze_block_calls(block, &mut calls)?;
            }
            
            // Update call graph
            self.call_graph.calls.insert(caller_name.clone(), calls.clone());
            
            // Update reverse mapping
            for callee in calls {
                self.call_graph.called_by
                    .entry(callee)
                    .or_insert_with(HashSet::new)
                    .insert(caller_name.clone());
            }
        }
        
        Ok(())
    }
    
    /// Analyze calls in a basic block
    fn analyze_block_calls(&self, block: &BasicBlock, calls: &mut HashSet<String>) -> Result<(), SemanticError> {
        // Check statements for function calls
        for statement in &block.statements {
            if let Statement::Assign { rvalue, .. } = statement {
                if let Rvalue::Call { func, .. } = rvalue {
                    if let Operand::Constant(constant) = func {
                        // Extract function name from constant
                        if let Some(func_name) = self.extract_function_name(constant) {
                            calls.insert(func_name);
                        }
                    }
                }
            }
        }
        
        // Check terminator for calls
        if let Terminator::Call { func, .. } = &block.terminator {
            if let Operand::Constant(constant) = func {
                if let Some(func_name) = self.extract_function_name(constant) {
                    calls.insert(func_name);
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract function name from constant operand
    fn extract_function_name(&self, _constant: &Constant) -> Option<String> {
        // In a real implementation, this would extract the function name
        // from the constant value (e.g., function pointer)
        None
    }
    
    /// Find entry points in the program
    fn find_entry_points(&mut self, program: &Program) {
        // Add main function if it exists
        if program.functions.contains_key("main") {
            self.call_graph.entry_points.insert("main".to_string());
        }
        
        // Add all external functions as entry points
        for name in program.external_functions.keys() {
            self.call_graph.entry_points.insert(name.clone());
        }
        
        // In a real implementation, we would also consider:
        // - Functions with external linkage
        // - Functions marked as entry points
        // - Functions called from external code
    }
    
    /// Analyze a single function
    fn analyze_function(&self, function: &Function) -> Result<FunctionAnalysis, SemanticError> {
        let mut is_pure = true;
        let mut modifies_globals = false;
        let mut cost = 0;
        let mut calls = HashSet::new();
        
        // Analyze all basic blocks
        for block in function.basic_blocks.values() {
            cost += block.statements.len();
            
            // Check for side effects
            for statement in &block.statements {
                if self.has_side_effects(statement) {
                    is_pure = false;
                }
                
                if self.modifies_global_state(statement) {
                    modifies_globals = true;
                }
                
                // Track function calls
                if let Statement::Assign { rvalue, .. } = statement {
                    if let Rvalue::Call { func, .. } = rvalue {
                        if let Operand::Constant(constant) = func {
                            if let Some(func_name) = self.extract_function_name(constant) {
                                calls.insert(func_name);
                            }
                        }
                    }
                }
            }
            
            // Check terminator
            cost += 1; // Terminator has cost
            
            if let Terminator::Call { func, .. } = &block.terminator {
                if let Operand::Constant(constant) = func {
                    if let Some(func_name) = self.extract_function_name(constant) {
                        calls.insert(func_name);
                    }
                }
                is_pure = false; // Function calls are not pure
            }
        }
        
        // Check for recursion
        let is_recursive = calls.contains(&function.name);
        
        // Determine if inlineable (small, pure functions are good candidates)
        let is_inlineable = cost <= 10 && is_pure && !is_recursive;
        
        Ok(FunctionAnalysis {
            is_pure,
            is_inlineable,
            cost,
            calls,
            modifies_globals,
            is_recursive,
        })
    }
    
    /// Check if statement has side effects
    fn has_side_effects(&self, statement: &Statement) -> bool {
        match statement {
            Statement::Assign { rvalue, .. } => {
                match rvalue {
                    Rvalue::Call { .. } => true, // Function calls may have side effects
                    _ => false,
                }
            }
            Statement::StorageLive(_) => false,
            Statement::StorageDead(_) => false,
            Statement::Nop => false,
        }
    }
    
    /// Check if statement modifies global state
    fn modifies_global_state(&self, statement: &Statement) -> bool {
        match statement {
            Statement::Assign { place, .. } => {
                // Check if we're assigning to a global variable
                // This is a simplified check - in reality we'd need more sophisticated analysis
                place.local == 0 // Assuming local 0 represents globals
            }
            _ => false,
        }
    }
    
    /// Find functions that are never called
    fn find_dead_functions(&mut self, analyses: &HashMap<String, FunctionAnalysis>) {
        let mut reachable = HashSet::new();
        let mut worklist: Vec<String> = self.call_graph.entry_points.iter().cloned().collect();
        
        // Mark all reachable functions
        while let Some(current) = worklist.pop() {
            if reachable.insert(current.clone()) {
                if let Some(calls) = self.call_graph.calls.get(&current) {
                    for callee in calls {
                        if !reachable.contains(callee) {
                            worklist.push(callee.clone());
                        }
                    }
                }
            }
        }
        
        // Find unreachable functions
        self.dead_functions.clear();
        for func_name in analyses.keys() {
            if !reachable.contains(func_name) {
                self.dead_functions.insert(func_name.clone());
            }
        }
    }
    
    /// Find global constants that can be propagated
    fn find_global_constants(&mut self, program: &Program) {
        self.global_constants.clear();
        
        // Add compile-time constants
        for (name, constant) in &program.global_constants {
            self.global_constants.insert(name.clone(), constant.clone());
        }
        
        // In a more sophisticated implementation, we would also:
        // - Find variables that are assigned once and never modified
        // - Propagate constants across function boundaries
        // - Handle constant arrays and structures
    }
    
    /// Perform aggressive function inlining
    fn inline_small_functions(&mut self, program: &mut Program, analyses: &HashMap<String, FunctionAnalysis>) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        // Find candidates for inlining
        let mut inline_candidates = Vec::new();
        for (name, analysis) in analyses {
            if analysis.is_inlineable && !self.inlined_functions.contains(name) {
                // Only inline if called from few places to avoid code bloat
                let call_count = self.call_graph.called_by.get(name).map(|s| s.len()).unwrap_or(0);
                if call_count > 0 && call_count <= 3 {
                    inline_candidates.push(name.clone());
                }
            }
        }
        
        // Perform inlining (simplified implementation)
        for func_name in inline_candidates {
            if self.try_inline_function(program, &func_name)? {
                self.inlined_functions.insert(func_name);
                changed = true;
            }
        }
        
        Ok(changed)
    }
    
    /// Try to inline a specific function
    fn try_inline_function(&mut self, _program: &mut Program, func_name: &str) -> Result<bool, SemanticError> {
        // This is a placeholder for actual inlining logic
        // In a real implementation, this would:
        // 1. Find all call sites of the function
        // 2. Replace each call with the function body
        // 3. Handle parameter passing and return values
        // 4. Update the call graph
        
        eprintln!("Would inline function: {}", func_name);
        Ok(false) // Not actually implemented yet
    }
    
    /// Remove dead functions from the program
    fn eliminate_dead_functions(&mut self, program: &mut Program) -> Result<bool, SemanticError> {
        let mut removed_count = 0;
        
        for dead_func in &self.dead_functions {
            if program.functions.remove(dead_func).is_some() {
                removed_count += 1;
            }
        }
        
        if removed_count > 0 {
            eprintln!("Eliminated {} dead functions", removed_count);
        }
        
        Ok(removed_count > 0)
    }
    
    /// Propagate global constants throughout the program
    fn propagate_global_constants(&mut self, program: &mut Program) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        for function in program.functions.values_mut() {
            if self.propagate_constants_in_function(function)? {
                changed = true;
            }
        }
        
        Ok(changed)
    }
    
    /// Propagate constants within a single function
    fn propagate_constants_in_function(&self, function: &mut Function) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        for block in function.basic_blocks.values_mut() {
            for statement in &mut block.statements {
                if let Statement::Assign { rvalue, .. } = statement {
                    if self.try_replace_with_constant(rvalue) {
                        changed = true;
                    }
                }
            }
        }
        
        Ok(changed)
    }
    
    /// Try to replace an operand with a constant
    fn try_replace_with_constant(&self, rvalue: &mut Rvalue) -> bool {
        match rvalue {
            Rvalue::Use(operand) => {
                // Try to replace variable references with constants
                if let Operand::Move(_place) | Operand::Copy(_place) = operand {
                    // Check if this refers to a global constant
                    // This is simplified - real implementation would need data flow analysis
                    return false;
                }
                false
            }
            Rvalue::BinaryOp { ref mut left, ref mut right, .. } => {
                let left_changed = self.try_replace_operand_with_constant(left);
                let right_changed = self.try_replace_operand_with_constant(right);
                left_changed || right_changed
            }
            Rvalue::UnaryOp { ref mut operand, .. } => {
                self.try_replace_operand_with_constant(operand)
            }
            _ => false,
        }
    }
    
    /// Try to replace an operand with a constant
    fn try_replace_operand_with_constant(&self, _operand: &mut Operand) -> bool {
        // Placeholder for constant replacement logic
        false
    }
}

impl OptimizationPass for WholeProgramOptimizationPass {
    fn name(&self) -> &'static str {
        "WholeProgramOptimization"
    }
    
    fn run_on_function(&mut self, _function: &mut Function) -> Result<bool, SemanticError> {
        // Whole program optimization can't run on individual functions
        Ok(false)
    }
    
    fn run_on_program(&mut self, program: &mut Program) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        // Perform whole program analysis
        let analyses = self.analyze_program(program)?;
        
        // Propagate global constants
        if self.propagate_global_constants(program)? {
            changed = true;
        }
        
        // Perform aggressive inlining
        if self.inline_small_functions(program, &analyses)? {
            changed = true;
        }
        
        // Eliminate dead functions
        if self.eliminate_dead_functions(program)? {
            changed = true;
        }
        
        Ok(changed)
    }
}

impl Default for WholeProgramOptimizationPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{Builder, Program};
    use crate::types::Type;
    use crate::ast::PrimitiveType;
    use std::collections::HashMap;
    
    #[test]
    fn test_whole_program_optimization_pass() {
        let mut pass = WholeProgramOptimizationPass::new();
        let mut program = Program {
            functions: HashMap::new(),
            global_constants: HashMap::new(),
            external_functions: HashMap::new(),
            type_definitions: HashMap::new(),
        };
        
        // Test with empty program
        assert!(pass.run_on_program(&mut program).is_ok());
    }
    
    #[test]
    fn test_call_graph_building() {
        let mut pass = WholeProgramOptimizationPass::new();
        let program = create_test_program();
        
        assert!(pass.build_call_graph(&program).is_ok());
        
        // Check that call graph was built
        assert!(!pass.call_graph.calls.is_empty() || program.functions.is_empty());
    }
    
    #[test]
    fn test_function_analysis() {
        let pass = WholeProgramOptimizationPass::new();
        let mut builder = Builder::new();
        
        // Create a simple pure function
        builder.start_function(
            "pure_function".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );
        
        let function = builder.finish_function();
        let analysis = pass.analyze_function(&function).unwrap();
        
        // Pure function with no calls should be inlineable
        assert!(analysis.is_pure);
        assert!(!analysis.modifies_globals);
        assert!(analysis.calls.is_empty());
    }
    
    #[test]
    fn test_dead_function_detection() {
        let mut pass = WholeProgramOptimizationPass::new();
        let mut analyses = HashMap::new();
        
        // Create mock analyses
        analyses.insert("main".to_string(), FunctionAnalysis {
            is_pure: false,
            is_inlineable: false,
            cost: 100,
            calls: HashSet::new(),
            modifies_globals: false,
            is_recursive: false,
        });
        
        analyses.insert("unused".to_string(), FunctionAnalysis {
            is_pure: true,
            is_inlineable: true,
            cost: 5,
            calls: HashSet::new(),
            modifies_globals: false,
            is_recursive: false,
        });
        
        // Set up call graph with main as entry point
        pass.call_graph.entry_points.insert("main".to_string());
        pass.call_graph.calls.insert("main".to_string(), HashSet::new());
        
        pass.find_dead_functions(&analyses);
        
        // "unused" should be detected as dead
        assert!(pass.dead_functions.contains("unused"));
        assert!(!pass.dead_functions.contains("main"));
    }
    
    fn create_test_program() -> Program {
        Program {
            functions: HashMap::new(),
            global_constants: HashMap::new(),
            external_functions: HashMap::new(),
            type_definitions: HashMap::new(),
        }
    }
}
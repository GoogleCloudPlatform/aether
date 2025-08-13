//! Interprocedural analysis and optimization for AetherScript
//!
//! Performs analysis and optimizations that span multiple functions,
//! including global constant propagation, escape analysis, and side effect analysis.

use crate::mir::{Function, Program, BasicBlock, Statement, Rvalue, Operand, Place, Terminator, Constant};
use crate::error::SemanticError;
use crate::optimizations::OptimizationPass;
use std::collections::{HashMap, HashSet};

/// Interprocedural analysis pass
#[derive(Debug)]
pub struct InterproceduralAnalysisPass {
    /// Call graph for the whole program
    call_graph: CallGraph,
    
    /// Function summaries
    summaries: HashMap<String, FunctionSummary>,
    
    /// Global constant analysis results
    global_constants: GlobalConstantAnalysis,
    
    /// Escape analysis results
    escape_analysis: EscapeAnalysis,
}

/// Call graph representation
#[derive(Debug, Default)]
pub struct CallGraph {
    /// Functions and their callees
    callees: HashMap<String, HashSet<String>>,
    
    /// Functions and their callers
    callers: HashMap<String, HashSet<String>>,
    
    /// Strongly connected components (for recursion detection)
    sccs: Vec<Vec<String>>,
    
    /// Topological ordering of functions
    topo_order: Vec<String>,
}

/// Side effect analysis
#[derive(Debug, Default)]
pub struct SideEffectAnalysis {
    /// Functions that are pure (no side effects)
    pure_functions: HashSet<String>,
    
    /// Functions that only read global state
    readonly_functions: HashSet<String>,
    
    /// Functions that modify global state
    modifying_functions: HashSet<String>,
    
    /// Functions that may throw exceptions
    throwing_functions: HashSet<String>,
    
    /// Functions that may not terminate
    nonterminating_functions: HashSet<String>,
}

/// Escape analysis
#[derive(Debug, Default)]
pub struct EscapeAnalysis {
    /// Variables that escape their function scope
    escaping_variables: HashMap<String, HashSet<Place>>,
    
    /// Variables that are only used locally
    local_variables: HashMap<String, HashSet<Place>>,
    
    /// Variables passed to other functions
    passed_variables: HashMap<String, HashSet<Place>>,
    
    /// Variables returned from functions
    returned_variables: HashMap<String, HashSet<Place>>,
}

/// Global constant propagation analysis
#[derive(Debug, Default)]
pub struct GlobalConstantAnalysis {
}

/// Alias analysis
#[derive(Debug, Default)]
pub struct AliasAnalysis {
}

/// Points-to analysis
#[derive(Debug, Default)]
pub struct PointsToAnalysis {
}

/// Abstract memory location for points-to analysis
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AbstractLocation {
    /// Function parameter
    Parameter(String, usize),
    
    /// Local variable
    Local(String, usize),
    
    /// Global variable
    Global(String),
    
    /// Heap allocation site
    HeapObject(String, usize),
    
    /// Return value of function
    ReturnValue(String),
    
    /// Unknown/external location
    Unknown,
}

/// Function summary for interprocedural analysis
#[derive(Debug, Clone)]
pub struct FunctionSummary {
    /// Function name
    pub name: String,
    
    /// Side effects performed by this function
    pub side_effects: SideEffectSummary,
    
    /// Parameters that escape
    pub escaping_parameters: HashSet<usize>,
    
    /// Global variables read
    pub reads_globals: HashSet<String>,
    
    /// Global variables modified
    pub modifies_globals: HashSet<String>,
    
    /// Functions called (direct calls only)
    pub calls: HashSet<String>,
    
    /// Whether function may not terminate
    pub may_not_terminate: bool,
    
    /// Whether function is recursive
    pub is_recursive: bool,
}

/// Summary of side effects for a function
#[derive(Debug, Clone, Default)]
pub struct SideEffectSummary {
    /// Reads from memory
    pub reads_memory: bool,
    
    /// Writes to memory
    pub writes_memory: bool,
    
    /// Performs I/O operations
    pub performs_io: bool,
    
    /// May throw exceptions
    pub may_throw: bool,
    
    /// Calls other functions
    pub calls_functions: bool,
}

impl InterproceduralAnalysisPass {
    pub fn new() -> Self {
        Self {
            call_graph: CallGraph::default(),
            summaries: HashMap::new(),
            global_constants: GlobalConstantAnalysis::default(),
            escape_analysis: EscapeAnalysis::default(),
        }
    }
    
    /// Perform complete interprocedural analysis
    pub fn analyze_program(&mut self, program: &Program) -> Result<HashMap<String, FunctionSummary>, SemanticError> {
        // Step 1: Build call graph
        self.build_call_graph(program)?;
        
        // Step 2: Perform side effect analysis
        let summaries = self.analyze_side_effects(program)?;
        
        // Step 3: Perform escape analysis
        self.analyze_escapes(program, &summaries)?;
        
        // Step 4: Perform alias analysis
        self.analyze_aliases(program, &summaries)?;
        
        // Step 5: Perform points-to analysis
        self.analyze_points_to(program, &summaries)?;
        
        // Step 6: Analyze global constants
        self.analyze_global_constants(program, &summaries)?;
        
        Ok(summaries)
    }
    
    /// Build the call graph for the program
    fn build_call_graph(&mut self, program: &Program) -> Result<(), SemanticError> {
        self.call_graph = CallGraph::default();
        
        // Initialize with all functions
        for function_name in program.functions.keys() {
            self.call_graph.callees.insert(function_name.clone(), HashSet::new());
            self.call_graph.callers.insert(function_name.clone(), HashSet::new());
        }
        
        // Find all function calls
        for (caller_name, function) in &program.functions {
            let mut callees = HashSet::new();
            
            for block in function.basic_blocks.values() {
                self.find_calls_in_block(block, &mut callees)?;
            }
            
            // Update call graph
            for callee in &callees {
                self.call_graph.callees.get_mut(caller_name).unwrap().insert(callee.clone());
                self.call_graph.callers.get_mut(callee).unwrap().insert(caller_name.clone());
            }
        }
        
        // Compute strongly connected components
        self.compute_sccs()?;
        
        // Compute topological ordering
        self.compute_topological_order()?;
        
        Ok(())
    }
    
    /// Find function calls in a basic block
    fn find_calls_in_block(&self, block: &BasicBlock, callees: &mut HashSet<String>) -> Result<(), SemanticError> {
        // Check statements for function calls
        for statement in &block.statements {
            if let Statement::Assign { rvalue, .. } = statement {
                if let Rvalue::Call { func, .. } = rvalue {
                    if let Some(func_name) = self.extract_function_name_from_operand(func) {
                        callees.insert(func_name);
                    }
                }
            }
        }
        
        // Check terminator for calls
        if let Terminator::Call { func, .. } = &block.terminator {
            if let Some(func_name) = self.extract_function_name_from_operand(func) {
                callees.insert(func_name);
            }
        }
        
        Ok(())
    }
    
    /// Extract function name from operand
    fn extract_function_name_from_operand(&self, operand: &Operand) -> Option<String> {
        // In a real implementation, this would extract the function name
        // from the operand (e.g., function constant)
        match operand {
            Operand::Constant(_constant) => {
                // Extract function name from constant
                // This is a placeholder implementation
                None
            }
            _ => None,
        }
    }
    
    /// Compute strongly connected components for recursion detection
    fn compute_sccs(&mut self) -> Result<(), SemanticError> {
        // Tarjan's algorithm for SCC computation
        let mut index = 0;
        let mut stack = Vec::new();
        let mut indices = HashMap::new();
        let mut lowlinks = HashMap::new();
        let mut on_stack = HashSet::new();
        let mut sccs = Vec::new();
        
        for function_name in self.call_graph.callees.keys() {
            if !indices.contains_key(function_name) {
                self.tarjan_scc(
                    function_name,
                    &mut index,
                    &mut stack,
                    &mut indices,
                    &mut lowlinks,
                    &mut on_stack,
                    &mut sccs,
                )?;
            }
        }
        
        self.call_graph.sccs = sccs;
        Ok(())
    }
    
    /// Tarjan's SCC algorithm helper
    fn tarjan_scc(
        &self,
        v: &str,
        index: &mut usize,
        stack: &mut Vec<String>,
        indices: &mut HashMap<String, usize>,
        lowlinks: &mut HashMap<String, usize>,
        on_stack: &mut HashSet<String>,
        sccs: &mut Vec<Vec<String>>,
    ) -> Result<(), SemanticError> {
        indices.insert(v.to_string(), *index);
        lowlinks.insert(v.to_string(), *index);
        *index += 1;
        stack.push(v.to_string());
        on_stack.insert(v.to_string());
        
        if let Some(successors) = self.call_graph.callees.get(v) {
            for w in successors {
                if !indices.contains_key(w) {
                    self.tarjan_scc(w, index, stack, indices, lowlinks, on_stack, sccs)?;
                    let w_lowlink = lowlinks[w];
                    let v_lowlink = lowlinks[v];
                    lowlinks.insert(v.to_string(), v_lowlink.min(w_lowlink));
                } else if on_stack.contains(w) {
                    let w_index = indices[w];
                    let v_lowlink = lowlinks[v];
                    lowlinks.insert(v.to_string(), v_lowlink.min(w_index));
                }
            }
        }
        
        if lowlinks[v] == indices[v] {
            let mut component = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                component.push(w.clone());
                if w == v {
                    break;
                }
            }
            sccs.push(component);
        }
        
        Ok(())
    }
    
    /// Compute topological ordering of functions
    fn compute_topological_order(&mut self) -> Result<(), SemanticError> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut order = Vec::new();
        
        for function_name in self.call_graph.callees.keys() {
            if !visited.contains(function_name) {
                self.topological_sort_visit(
                    function_name,
                    &mut visited,
                    &mut temp_visited,
                    &mut order,
                )?;
            }
        }
        
        order.reverse();
        self.call_graph.topo_order = order;
        Ok(())
    }
    
    /// Topological sort helper
    fn topological_sort_visit(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), SemanticError> {
        if temp_visited.contains(node) {
            // Cycle detected, but we'll handle it gracefully
            return Ok(());
        }
        
        if visited.contains(node) {
            return Ok(());
        }
        
        temp_visited.insert(node.to_string());
        
        if let Some(successors) = self.call_graph.callees.get(node) {
            for successor in successors {
                self.topological_sort_visit(successor, visited, temp_visited, order)?;
            }
        }
        
        temp_visited.remove(node);
        visited.insert(node.to_string());
        order.push(node.to_string());
        
        Ok(())
    }
    
    /// Analyze side effects of all functions
    fn analyze_side_effects(&mut self, program: &Program) -> Result<HashMap<String, FunctionSummary>, SemanticError> {
        let mut summaries = HashMap::new();
        
        // Process functions in reverse topological order (callees before callers)
        for function_name in self.call_graph.topo_order.iter().rev() {
            if let Some(function) = program.functions.get(function_name) {
                let summary = self.analyze_function_side_effects(function, &summaries)?;
                summaries.insert(function_name.clone(), summary);
            }
        }
        
        // Update global side effect analysis
        self.update_global_side_effect_analysis(&summaries);
        
        Ok(summaries)
    }
    
    /// Analyze side effects of a single function
    fn analyze_function_side_effects(
        &self,
        function: &Function,
        existing_summaries: &HashMap<String, FunctionSummary>,
    ) -> Result<FunctionSummary, SemanticError> {
        let mut summary = FunctionSummary {
            name: function.name.clone(),
            side_effects: SideEffectSummary::default(),
            escaping_parameters: HashSet::new(),
            reads_globals: HashSet::new(),
            modifies_globals: HashSet::new(),
            calls: HashSet::new(),
            may_not_terminate: false,
            is_recursive: false,
        };
        
        // Check if function is recursive
        if let Some(callees) = self.call_graph.callees.get(&function.name) {
            summary.is_recursive = callees.contains(&function.name);
        }
        
        // Analyze each basic block
        for block in function.basic_blocks.values() {
            self.analyze_block_side_effects(block, &mut summary, existing_summaries)?;
        }
        
        Ok(summary)
    }
    
    /// Analyze side effects in a basic block
    fn analyze_block_side_effects(
        &self,
        block: &BasicBlock,
        summary: &mut FunctionSummary,
        existing_summaries: &HashMap<String, FunctionSummary>,
    ) -> Result<(), SemanticError> {
        // Analyze statements
        for statement in &block.statements {
            self.analyze_statement_side_effects(statement, summary, existing_summaries)?;
        }
        
        // Analyze terminator
        self.analyze_terminator_side_effects(&block.terminator, summary, existing_summaries)?;
        
        Ok(())
    }
    
    /// Analyze side effects of a statement
    fn analyze_statement_side_effects(
        &self,
        statement: &Statement,
        summary: &mut FunctionSummary,
        existing_summaries: &HashMap<String, FunctionSummary>,
    ) -> Result<(), SemanticError> {
        match statement {
            Statement::Assign { place, rvalue, .. } => {
                // Check if we're modifying a global
                if self.is_global_place(place) {
                    summary.side_effects.writes_memory = true;
                    summary.modifies_globals.insert(self.extract_global_name(place));
                }
                
                // Analyze the rvalue
                self.analyze_rvalue_side_effects(rvalue, summary, existing_summaries)?;
            }
            _ => {
                // Other statements typically don't have side effects
            }
        }
        
        Ok(())
    }
    
    /// Analyze side effects of an rvalue
    fn analyze_rvalue_side_effects(
        &self,
        rvalue: &Rvalue,
        summary: &mut FunctionSummary,
        existing_summaries: &HashMap<String, FunctionSummary>,
    ) -> Result<(), SemanticError> {
        match rvalue {
            Rvalue::Use(operand) => {
                if let Operand::Move(place) | Operand::Copy(place) = operand {
                    if self.is_global_place(place) {
                        summary.side_effects.reads_memory = true;
                        summary.reads_globals.insert(self.extract_global_name(place));
                    }
                }
            }
            Rvalue::Call { func, args } => {
                summary.side_effects.calls_functions = true;
                
                // Extract function name and propagate its side effects
                if let Some(func_name) = self.extract_function_name_from_operand(func) {
                    summary.calls.insert(func_name.clone());
                    
                    if let Some(callee_summary) = existing_summaries.get(&func_name) {
                        self.propagate_side_effects(&callee_summary.side_effects, &mut summary.side_effects);
                        
                        // Propagate global reads/writes
                        for global in &callee_summary.reads_globals {
                            summary.reads_globals.insert(global.clone());
                        }
                        for global in &callee_summary.modifies_globals {
                            summary.modifies_globals.insert(global.clone());
                        }
                        
                        if callee_summary.may_not_terminate {
                            summary.may_not_terminate = true;
                        }
                    }
                }
                
                // Analyze arguments for global access
                for arg in args {
                    if let Operand::Move(place) | Operand::Copy(place) = arg {
                        if self.is_global_place(place) {
                            summary.side_effects.reads_memory = true;
                            summary.reads_globals.insert(self.extract_global_name(place));
                        }
                    }
                }
            }
            Rvalue::BinaryOp { left, right, .. } => {
                // Check operands for global access
                for operand in [left, right] {
                    if let Operand::Move(place) | Operand::Copy(place) = operand {
                        if self.is_global_place(place) {
                            summary.side_effects.reads_memory = true;
                            summary.reads_globals.insert(self.extract_global_name(place));
                        }
                    }
                }
            }
            Rvalue::UnaryOp { operand, .. } => {
                if let Operand::Move(place) | Operand::Copy(place) = operand {
                    if self.is_global_place(place) {
                        summary.side_effects.reads_memory = true;
                        summary.reads_globals.insert(self.extract_global_name(place));
                    }
                }
            }
            _ => {
                // Other rvalues typically don't have side effects
            }
        }
        
        Ok(())
    }
    
    /// Analyze side effects of a terminator
    fn analyze_terminator_side_effects(
        &self,
        terminator: &Terminator,
        summary: &mut FunctionSummary,
        existing_summaries: &HashMap<String, FunctionSummary>,
    ) -> Result<(), SemanticError> {
        match terminator {
            Terminator::Call { func,  .. } => {
                summary.side_effects.calls_functions = true;
                
                if let Some(func_name) = self.extract_function_name_from_operand(func) {
                    summary.calls.insert(func_name.clone());
                    
                    if let Some(callee_summary) = existing_summaries.get(&func_name) {
                        self.propagate_side_effects(&callee_summary.side_effects, &mut summary.side_effects);
                    }
                }
            }
            _ => {
                // Other terminators typically don't have side effects
            }
        }
        
        Ok(())
    }
    
    /// Propagate side effects from callee to caller
    fn propagate_side_effects(&self, from: &SideEffectSummary, to: &mut SideEffectSummary) {
        to.reads_memory |= from.reads_memory;
        to.writes_memory |= from.writes_memory;
        to.performs_io |= from.performs_io;
        to.may_throw |= from.may_throw;
        to.calls_functions |= from.calls_functions;
    }
    
    /// Check if a place refers to a global variable
    fn is_global_place(&self, place: &Place) -> bool {
        // Simplified check - assume local 0 is globals
        // In a real implementation, this would check the variable scope
        place.local == 0
    }
    
    /// Extract global variable name from place
    fn extract_global_name(&self, place: &Place) -> String {
        // Simplified extraction
        format!("global_{}", place.local)
    }
    
    /// Update global side effect analysis
    fn update_global_side_effect_analysis(&mut self, summaries: &HashMap<String, FunctionSummary>) {
        for (function_name, summary) in summaries {
            // In a real implementation, this would be populated from side effect analysis
            if !summary.side_effects.reads_memory && !summary.side_effects.writes_memory {
                // Pure function - no memory side effects
            } else if summary.side_effects.reads_memory && !summary.side_effects.writes_memory {
                // In a real implementation, this would be populated from side effect analysis
            } else if summary.side_effects.writes_memory {
                // In a real implementation, this would be populated from side effect analysis
            }
            
            if summary.side_effects.may_throw {
                // In a real implementation, this would be populated from side effect analysis
            }
            
            if summary.may_not_terminate {
                // In a real implementation, this would be populated from side effect analysis
            }
        }
    }
    
    /// Perform escape analysis
    fn analyze_escapes(&mut self, program: &Program, _summaries: &HashMap<String, FunctionSummary>) -> Result<(), SemanticError> {
        // For each function, determine which variables escape
        for (function_name, function) in &program.functions {
            let mut escaping = HashSet::new();
            let mut local = HashSet::new();
            let mut passed = HashSet::new();
            let mut returned = HashSet::new();
            
            // Analyze function body
            self.analyze_function_escapes(function, &mut escaping, &mut local, &mut passed, &mut returned)?;
            
            self.escape_analysis.escaping_variables.insert(function_name.clone(), escaping);
            self.escape_analysis.local_variables.insert(function_name.clone(), local);
            self.escape_analysis.passed_variables.insert(function_name.clone(), passed);
            self.escape_analysis.returned_variables.insert(function_name.clone(), returned);
        }
        
        Ok(())
    }
    
    /// Analyze escapes in a single function
    fn analyze_function_escapes(
        &self,
        function: &Function,
        escaping: &mut HashSet<Place>,
        local: &mut HashSet<Place>,
        passed: &mut HashSet<Place>,
        _returned: &mut HashSet<Place>,
    ) -> Result<(), SemanticError> {
        // This is a simplified escape analysis
        // A real implementation would be much more sophisticated
        
        for block in function.basic_blocks.values() {
            for statement in &block.statements {
                match statement {
                    Statement::Assign { place, rvalue, .. } => {
                        match rvalue {
                            Rvalue::Call { args, .. } => {
                                // Arguments to function calls may escape
                                for arg in args {
                                    if let Operand::Move(arg_place) | Operand::Copy(arg_place) = arg {
                                        escaping.insert(arg_place.clone());
                                        passed.insert(arg_place.clone());
                                    }
                                }
                            }
                            _ => {
                                // Local assignment
                                local.insert(place.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            // Check terminator for returns
            if let Terminator::Return = &block.terminator {
                // The return value escapes
                // In a real implementation, we'd track what's being returned
            }
        }
        
        Ok(())
    }
    
    /// Perform alias analysis
    fn analyze_aliases(&mut self, program: &Program, _summaries: &HashMap<String, FunctionSummary>) -> Result<(), SemanticError> {
        // This is a placeholder for alias analysis
        // Real alias analysis is quite complex and would require sophisticated algorithms
        
        for (function_name, function) in &program.functions {
            // Simple alias analysis - assume no aliasing for now
            // In reality, this would use algorithms like Andersen's or Steensgaard's
        }
        
        Ok(())
    }
    
    /// Perform points-to analysis
    fn analyze_points_to(&mut self, program: &Program, _summaries: &HashMap<String, FunctionSummary>) -> Result<(), SemanticError> {
        // This is a placeholder for points-to analysis
        // Real points-to analysis would track what each pointer variable can point to
        
        for (function_name, function) in &program.functions {
            // Initialize points-to sets for all pointer variables
            for block in function.basic_blocks.values() {
                for _statement in &block.statements {
                    // Analyze pointer assignments and dereferences
                }
            }
        }
        
        Ok(())
    }
    
    /// Analyze global constants
    fn analyze_global_constants(&mut self, program: &Program, _summaries: &HashMap<String, FunctionSummary>) -> Result<(), SemanticError> {
        // Start with program-level constants
        // In a real implementation, this would be populated from global constant analysis
        
        // Find variables that are effectively constant
        for (function_name, function) in &program.functions {
            self.find_constant_variables(function)?;
        }
        
        Ok(())
    }
    
    /// Find variables that are effectively constant
    fn find_constant_variables(&mut self, function: &Function) -> Result<(), SemanticError> {
        // Look for variables that are assigned once and never modified
        let mut assignments = HashMap::new();
        let mut modifications = HashSet::new();
        
        for block in function.basic_blocks.values() {
            for statement in &block.statements {
                if let Statement::Assign { place, rvalue, .. } = statement {
                    if let Rvalue::Use(Operand::Constant(constant)) = rvalue {
                        // This is a constant assignment
                        if !assignments.contains_key(place) && !modifications.contains(place) {
                            assignments.insert(place.clone(), constant.clone());
                        } else {
                            // Multiple assignments, not constant
                            assignments.remove(place);
                            modifications.insert(place.clone());
                        }
                    } else {
                        // Non-constant assignment
                        assignments.remove(place);
                        modifications.insert(place.clone());
                    }
                }
            }
        }
        
        // Variables in `assignments` are effectively constant
        for (place, constant) in assignments {
            let var_name = format!("{}::{}", function.name, place.local);
        }
        
        Ok(())
    }
    
    /// Apply interprocedural optimizations
    pub fn apply_optimizations(&self, program: &mut Program, summaries: &HashMap<String, FunctionSummary>) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        // Apply constant propagation
        if self.apply_global_constant_propagation(program)? {
            changed = true;
        }
        
        // Apply dead function elimination
        if self.eliminate_dead_functions(program, summaries)? {
            changed = true;
        }
        
        // Apply pure function optimizations
        if self.optimize_pure_functions(program, summaries)? {
            changed = true;
        }
        
        Ok(changed)
    }
    
    /// Apply global constant propagation
    fn apply_global_constant_propagation(&self, program: &mut Program) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        for function in program.functions.values_mut() {
            if self.propagate_constants_in_function(function)? {
                changed = true;
            }
        }
        
        Ok(changed)
    }
    
    /// Propagate constants within a function
    fn propagate_constants_in_function(&self, function: &mut Function) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        for block in function.basic_blocks.values_mut() {
            for statement in &mut block.statements {
                if let Statement::Assign { rvalue, .. } = statement {
                    if self.try_replace_with_global_constant(rvalue) {
                        changed = true;
                    }
                }
            }
        }
        
        Ok(changed)
    }
    
    /// Try to replace operands with global constants
    fn try_replace_with_global_constant(&self, _rvalue: &mut Rvalue) -> bool {
        // This would replace variable references with known constants
        // Placeholder implementation
        false
    }
    
    /// Eliminate functions that are never called
    fn eliminate_dead_functions(&self, program: &mut Program, _summaries: &HashMap<String, FunctionSummary>) -> Result<bool, SemanticError> {
        let mut dead_functions = Vec::new();
        
        // Find functions with no callers (except entry points like main)
        for function_name in program.functions.keys() {
            if function_name != "main" && 
               !program.external_functions.contains_key(function_name) {
                if let Some(callers) = self.call_graph.callers.get(function_name) {
                    if callers.is_empty() {
                        dead_functions.push(function_name.clone());
                    }
                }
            }
        }
        
        // Remove dead functions
        let mut removed_count = 0;
        for dead_func in dead_functions {
            program.functions.remove(&dead_func);
            removed_count += 1;
        }
        
        if removed_count > 0 {
            eprintln!("Eliminated {} dead functions via interprocedural analysis", removed_count);
        }
        
        Ok(removed_count > 0)
    }
    
    /// Optimize calls to pure functions
    fn optimize_pure_functions(&self, program: &mut Program, summaries: &HashMap<String, FunctionSummary>) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        // Pure functions with constant arguments can be evaluated at compile time
        for function in program.functions.values_mut() {
            if self.optimize_pure_calls_in_function(function, summaries)? {
                changed = true;
            }
        }
        
        Ok(changed)
    }
    
    /// Optimize pure function calls within a function
    fn optimize_pure_calls_in_function(&self, _function: &mut Function, _summaries: &HashMap<String, FunctionSummary>) -> Result<bool, SemanticError> {
        // This would identify calls to pure functions with constant arguments
        // and replace them with the computed result
        // Placeholder implementation
        Ok(false)
    }
}

impl OptimizationPass for InterproceduralAnalysisPass {
    fn name(&self) -> &'static str {
        "InterproceduralAnalysis"
    }
    
    fn run_on_function(&mut self, _function: &mut Function) -> Result<bool, SemanticError> {
        // Interprocedural analysis requires the whole program
        Ok(false)
    }
    
    fn run_on_program(&mut self, program: &mut Program) -> Result<bool, SemanticError> {
        // Perform interprocedural analysis
        let summaries = self.analyze_program(program)?;
        
        // Apply optimizations based on the analysis
        self.apply_optimizations(program, &summaries)
    }
}

impl Default for InterproceduralAnalysisPass {
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
    fn test_interprocedural_analysis_pass() {
        let pass = InterproceduralAnalysisPass::new();
        assert_eq!(pass.name(), "InterproceduralAnalysis");
        assert!(pass.call_graph.callees.is_empty());
    }
    
    #[test]
    fn test_call_graph_building() {
        let mut pass = InterproceduralAnalysisPass::new();
        let program = create_test_program();
        
        assert!(pass.build_call_graph(&program).is_ok());
        
        // Should have initialized call graph for all functions
        for function_name in program.functions.keys() {
            assert!(pass.call_graph.callees.contains_key(function_name));
            assert!(pass.call_graph.callers.contains_key(function_name));
        }
    }
    
    #[test]
    fn test_side_effect_analysis() {
        let mut pass = InterproceduralAnalysisPass::new();
        let program = create_test_program();
        
        pass.build_call_graph(&program).unwrap();
        let summaries = pass.analyze_side_effects(&program).unwrap();
        
        // Should have created summaries for all functions
        for function_name in program.functions.keys() {
            assert!(summaries.contains_key(function_name));
        }
    }
    
    #[test]
    fn test_function_summary() {
        let summary = FunctionSummary {
            name: "test_function".to_string(),
            side_effects: SideEffectSummary {
                reads_memory: true,
                writes_memory: false,
                performs_io: false,
                may_throw: false,
                calls_functions: false,
            },
            escaping_parameters: HashSet::new(),
            reads_globals: HashSet::new(),
            modifies_globals: HashSet::new(),
            calls: HashSet::new(),
            may_not_terminate: false,
            is_recursive: false,
        };
        
        assert_eq!(summary.name, "test_function");
        assert!(summary.side_effects.reads_memory);
        assert!(!summary.side_effects.writes_memory);
        assert!(!summary.is_recursive);
    }
    
    #[test]
    fn test_abstract_location() {
        let param_loc = AbstractLocation::Parameter("function".to_string(), 0);
        let global_loc = AbstractLocation::Global("global_var".to_string());
        let unknown_loc = AbstractLocation::Unknown;
        
        assert_ne!(param_loc, global_loc);
        assert_ne!(param_loc, unknown_loc);
    }
    
    fn create_test_program() -> Program {
        let mut program = Program {
            functions: HashMap::new(),
            global_constants: HashMap::new(),
            external_functions: HashMap::new(),
            type_definitions: HashMap::new(),
        };
        
        // Create a simple test function
        let mut builder = Builder::new();
        builder.start_function(
            "test_function".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );
        let function = builder.finish_function();
        
        program.functions.insert("test_function".to_string(), function);
        program
    }
}
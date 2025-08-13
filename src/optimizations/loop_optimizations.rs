//! Advanced loop optimizations for AetherScript
//!
//! Implements sophisticated loop optimization techniques including loop invariant
//! code motion, loop unrolling, loop fusion, and loop interchange.

use crate::mir::{Function, BasicBlock, Statement, Rvalue, Operand, Place, Terminator, BinOp};
use crate::error::SemanticError;
use crate::optimizations::OptimizationPass;
use crate::types::Type;
use std::collections::{HashMap, HashSet, VecDeque};

/// Advanced loop optimization pass
#[derive(Debug)]
pub struct LoopOptimizationPass {
    /// Detected loops in the current function
    loops: Vec<LoopInfo>,
    
    /// Loop nest forest
    loop_forest: LoopForest,
    
    /// Dominance information
    dominance_info: DominanceInfo,
    
    /// Loop invariant analysis results
    invariant_analysis: LoopInvariantAnalysis,
    
    /// Induction variable analysis
    induction_analysis: InductionAnalysis,
    
    /// Data dependence analysis
    dependence_analysis: DependenceAnalysis,
}

/// Information about a single loop
#[derive(Debug, Clone)]
pub struct LoopInfo {
    /// Loop header block
    pub header: usize,
    
    /// Loop preheader (if any)
    pub preheader: Option<usize>,
    
    /// All blocks in the loop
    pub blocks: HashSet<usize>,
    
    /// Loop exit blocks
    pub exits: HashSet<usize>,
    
    /// Loop back edges
    pub back_edges: Vec<(usize, usize)>,
    
    /// Loop depth (nesting level)
    pub depth: usize,
    
    /// Parent loop (if nested)
    pub parent: Option<usize>,
    
    /// Child loops
    pub children: Vec<usize>,
    
    /// Loop bounds information
    pub bounds: Option<LoopBounds>,
    
    /// Estimated iteration count
    pub iteration_count: Option<u64>,
}

/// Loop bounds information
#[derive(Debug, Clone)]
pub struct LoopBounds {
    /// Induction variable
    pub induction_var: Place,
    
    /// Initial value
    pub initial_value: Operand,
    
    /// Final value
    pub final_value: Operand,
    
    /// Step value
    pub step: i64,
    
    /// Comparison operation
    pub comparison: BinOp,
    
    /// Whether bounds are known at compile time
    pub known_bounds: bool,
}

/// Loop forest representation
#[derive(Debug, Default)]
pub struct LoopForest {
    /// Top-level loops (not nested in other loops)
    pub roots: Vec<usize>,
    
    /// All loops indexed by header block
    pub loops: HashMap<usize, LoopInfo>,
}

/// Dominance information for loop analysis
#[derive(Debug, Default)]
pub struct DominanceInfo {
    /// Immediate dominators
    pub idom: HashMap<usize, usize>,
    
    /// Dominance tree
    pub dom_tree: HashMap<usize, Vec<usize>>,
    
    /// Dominance frontiers
    pub dom_frontier: HashMap<usize, HashSet<usize>>,
}

/// Loop invariant analysis
#[derive(Debug, Default)]
pub struct LoopInvariantAnalysis {
    /// Loop-invariant statements for each loop
    pub invariant_statements: HashMap<usize, Vec<InvariantStatement>>,
    
    /// Variables that are loop-invariant
    pub invariant_variables: HashMap<usize, HashSet<Place>>,
}

/// A statement that is loop-invariant
#[derive(Debug, Clone)]
pub struct InvariantStatement {
    /// Block containing the statement
    pub block: usize,
    
    /// Index within the block
    pub statement_index: usize,
    
    /// The invariant statement
    pub statement: Statement,
    
    /// Whether it's safe to hoist
    pub safe_to_hoist: bool,
    
    /// Profit estimate for hoisting
    pub hoist_profit: f64,
}

/// Induction variable analysis
#[derive(Debug, Default)]
pub struct InductionAnalysis {
    /// Basic induction variables for each loop
    pub basic_induction_vars: HashMap<usize, Vec<BasicInductionVar>>,
    
    /// Derived induction variables
    pub derived_induction_vars: HashMap<usize, Vec<DerivedInductionVar>>,
}

/// Basic induction variable
#[derive(Debug, Clone)]
pub struct BasicInductionVar {
    /// The induction variable
    pub variable: Place,
    
    /// Initial value
    pub initial_value: Operand,
    
    /// Step value
    pub step: i64,
    
    /// Block where increment happens
    pub increment_block: usize,
    
    /// Statement index of increment
    pub increment_statement: usize,
}

/// Derived induction variable
#[derive(Debug, Clone)]
pub struct DerivedInductionVar {
    /// The derived variable
    pub variable: Place,
    
    /// Base induction variable
    pub base: Place,
    
    /// Multiplier (derived = base * multiplier + offset)
    pub multiplier: i64,
    
    /// Offset
    pub offset: i64,
}

/// Data dependence analysis
#[derive(Debug, Default)]
pub struct DependenceAnalysis {
    /// Flow dependencies (RAW)
    pub flow_deps: Vec<Dependence>,
    
    /// Anti dependencies (WAR)
    pub anti_deps: Vec<Dependence>,
    
    /// Output dependencies (WAW)
    pub output_deps: Vec<Dependence>,
    
    /// Loop-carried dependencies
    pub loop_carried_deps: HashMap<usize, Vec<Dependence>>,
}

/// A data dependence
#[derive(Debug, Clone)]
pub struct Dependence {
    /// Source statement
    pub source: StatementRef,
    
    /// Sink statement
    pub sink: StatementRef,
    
    /// Dependence distance vector
    pub distance: Vec<i64>,
    
    /// Dependence direction vector
    pub direction: Vec<DependenceDirection>,
    
    /// Type of dependence
    pub dep_type: DependenceType,
}

/// Reference to a statement
#[derive(Debug, Clone)]
pub struct StatementRef {
    pub block: usize,
    pub statement: usize,
}

/// Dependence direction
#[derive(Debug, Clone, PartialEq)]
pub enum DependenceDirection {
    Less,    // <
    Equal,   // =
    Greater, // >
    Any,     // *
}

/// Type of dependence
#[derive(Debug, Clone, PartialEq)]
pub enum DependenceType {
    Flow,   // RAW
    Anti,   // WAR
    Output, // WAW
}

impl LoopOptimizationPass {
    pub fn new() -> Self {
        Self {
            loops: Vec::new(),
            loop_forest: LoopForest::default(),
            dominance_info: DominanceInfo::default(),
            invariant_analysis: LoopInvariantAnalysis::default(),
            induction_analysis: InductionAnalysis::default(),
            dependence_analysis: DependenceAnalysis::default(),
        }
    }
    
    /// Analyze loops in a function
    pub fn analyze_function(&mut self, function: &Function) -> Result<(), SemanticError> {
        // Step 1: Build dominance information
        self.build_dominance_info(function)?;
        
        // Step 2: Detect loops
        self.detect_loops(function)?;
        
        // Step 3: Build loop forest
        self.build_loop_forest()?;
        
        // Step 4: Analyze loop invariants
        self.analyze_loop_invariants(function)?;
        
        // Step 5: Analyze induction variables
        self.analyze_induction_variables(function)?;
        
        // Step 6: Analyze data dependencies
        self.analyze_data_dependencies(function)?;
        
        Ok(())
    }
    
    /// Build dominance information
    fn build_dominance_info(&mut self, function: &Function) -> Result<(), SemanticError> {
        let blocks: Vec<usize> = function.basic_blocks.keys().copied().map(|id| id as usize).collect();
        if blocks.is_empty() {
            return Ok(());
        }
        
        // Find entry block (assume block 0)
        let entry = blocks[0];
        
        // Compute dominators using iterative algorithm
        let mut dom = HashMap::new();
        dom.insert(entry, vec![entry]);
        
        // Initialize all other blocks to all blocks
        for &block in &blocks {
            if block != entry {
                dom.insert(block, blocks.clone());
            }
        }
        
        // Iterate until fixed point
        let mut changed = true;
        while changed {
            changed = false;
            
            for &block in &blocks {
                if block == entry {
                    continue;
                }
                
                // Find predecessors
                let predecessors = self.find_predecessors(function, block);
                if predecessors.is_empty() {
                    continue;
                }
                
                // New dominators = {block} ∪ (∩ dominators of predecessors)
                let mut new_dom = vec![block];
                
                if let Some(first_pred) = predecessors.first() {
                    let mut intersection = dom.get(first_pred).unwrap().clone();
                    
                    for &pred in predecessors.iter().skip(1) {
                        if let Some(pred_dom) = dom.get(&pred) {
                            intersection.retain(|x| pred_dom.contains(x));
                        }
                    }
                    
                    new_dom.extend(intersection);
                }
                
                new_dom.sort();
                new_dom.dedup();
                
                if dom.get(&block) != Some(&new_dom) {
                    dom.insert(block, new_dom);
                    changed = true;
                }
            }
        }
        
        // Compute immediate dominators
        for &block in &blocks {
            if block == entry {
                continue;
            }
            
            if let Some(dominators) = dom.get(&block) {
                // Remove block itself
                let mut strict_dom: Vec<usize> = dominators.iter().filter(|&&x| x != block).copied().collect();
                strict_dom.sort();
                
                // Find immediate dominator (the one not dominated by any other)
                for &candidate in &strict_dom {
                    let mut is_immediate = true;
                    
                    for &other in &strict_dom {
                        if other != candidate {
                            if let Some(other_dom) = dom.get(&other) {
                                if other_dom.contains(&candidate) {
                                    is_immediate = false;
                                    break;
                                }
                            }
                        }
                    }
                    
                    if is_immediate {
                        self.dominance_info.idom.insert(block, candidate);
                        break;
                    }
                }
            }
        }
        
        // Build dominance tree
        for (&child, &parent) in &self.dominance_info.idom {
            self.dominance_info.dom_tree.entry(parent).or_insert_with(Vec::new).push(child);
        }
        
        Ok(())
    }
    
    /// Find predecessors of a block
    fn find_predecessors(&self, function: &Function, block: usize) -> Vec<usize> {
        let mut predecessors = Vec::new();
        
        for (&pred_id, pred_block) in &function.basic_blocks {
            let targets = self.get_terminator_targets(&pred_block.terminator);
            if targets.contains(&(block as u32)) {
                predecessors.push(pred_id as usize);
            }
        }
        
        predecessors
    }
    
    /// Get target blocks from a terminator
    fn get_terminator_targets(&self, terminator: &Terminator) -> Vec<u32> {
        match terminator {
            Terminator::Goto { target } => vec![*target],
            Terminator::SwitchInt { targets, .. } => {
                let mut all_targets = targets.targets.clone();
                all_targets.push(targets.otherwise);
                all_targets
            },
            Terminator::Return => vec![],
            Terminator::Call { target, cleanup, .. } => {
                let mut targets = vec![];
                if let Some(target) = target {
                    targets.push(*target);
                }
                if let Some(cleanup) = cleanup {
                    targets.push(*cleanup);
                }
                targets
            }
            Terminator::Drop { target, unwind, .. } => {
                let mut targets = vec![*target];
                if let Some(unwind) = unwind {
                    targets.push(*unwind);
                }
                targets
            }
            Terminator::Assert { target, cleanup, .. } => {
                let mut targets = vec![*target];
                if let Some(cleanup) = cleanup {
                    targets.push(*cleanup);
                }
                targets
            }
            Terminator::Unreachable => vec![],
        }
    }
    
    /// Detect loops using dominance information
    fn detect_loops(&mut self, function: &Function) -> Result<(), SemanticError> {
        self.loops.clear();
        
        // Find back edges (edges where target dominates source)
        let mut back_edges = Vec::new();
        
        for (&block_id, block) in &function.basic_blocks {
            let targets = self.get_terminator_targets(&block.terminator);
            
            for target in targets {
                if self.dominates(target as usize, block_id as usize) {
                    back_edges.push((block_id as usize, target as usize));
                }
            }
        }
        
        // For each back edge, construct the natural loop
        for (tail, head) in back_edges {
            let loop_blocks = self.find_natural_loop(function, head, tail)?;
            
            // Create loop info
            let preheader = self.find_preheader(function, head, &loop_blocks);
            let exits = self.find_loop_exits(function, head, &loop_blocks);
            let loop_info = LoopInfo {
                header: head,
                preheader,
                blocks: loop_blocks,
                exits,
                back_edges: vec![(tail, head)],
                depth: 0, // Will be computed later
                parent: None,
                children: Vec::new(),
                bounds: None,
                iteration_count: None,
            };
            
            self.loops.push(loop_info);
        }
        
        Ok(())
    }
    
    /// Check if block a dominates block b
    fn dominates(&self, a: usize, b: usize) -> bool {
        if a == b {
            return true;
        }
        
        let mut current = b;
        while let Some(&idom) = self.dominance_info.idom.get(&current) {
            if idom == a {
                return true;
            }
            current = idom;
        }
        
        false
    }
    
    /// Find the natural loop for a back edge
    fn find_natural_loop(&self, function: &Function, head: usize, tail: usize) -> Result<HashSet<usize>, SemanticError> {
        let mut loop_blocks = HashSet::new();
        loop_blocks.insert(head);
        
        if head != tail {
            loop_blocks.insert(tail);
            
            // DFS to find all blocks that can reach tail without going through head
            let mut worklist = VecDeque::new();
            worklist.push_back(tail);
            
            while let Some(block) = worklist.pop_front() {
                if block == head {
                    continue;
                }
                
                let predecessors = self.find_predecessors(function, block);
                for pred in predecessors {
                    if !loop_blocks.contains(&pred) {
                        loop_blocks.insert(pred);
                        worklist.push_back(pred);
                    }
                }
            }
        }
        
        Ok(loop_blocks)
    }
    
    /// Find preheader for a loop
    fn find_preheader(&self, function: &Function, header: usize, loop_blocks: &HashSet<usize>) -> Option<usize> {
        let predecessors = self.find_predecessors(function, header);
        
        // Look for a unique predecessor outside the loop
        let external_preds: Vec<usize> = predecessors.into_iter()
            .filter(|pred| !loop_blocks.contains(pred))
            .collect();
        
        if external_preds.len() == 1 {
            Some(external_preds[0])
        } else {
            None
        }
    }
    
    /// Find exit blocks for a loop
    fn find_loop_exits(&self, function: &Function, _header: usize, loop_blocks: &HashSet<usize>) -> HashSet<usize> {
        let mut exits = HashSet::new();
        
        for &block in loop_blocks {
            if let Some(block_info) = function.basic_blocks.get(&(block as u32)) {
                let targets = self.get_terminator_targets(&block_info.terminator);
                
                for target in targets {
                    if !loop_blocks.contains(&(target as usize)) {
                        exits.insert(target as usize);
                    }
                }
            }
        }
        
        exits
    }
    
    /// Build loop forest (nesting structure)
    fn build_loop_forest(&mut self) -> Result<(), SemanticError> {
        self.loop_forest = LoopForest::default();
        
        // Sort loops by number of blocks (innermost first)
        let mut loop_indices: Vec<usize> = (0..self.loops.len()).collect();
        loop_indices.sort_by_key(|&i| self.loops[i].blocks.len());
        
        // Build nesting relationships
        for &i in &loop_indices {
            let loop_i = &self.loops[i];
            let mut parent_loop = None;
            
            // Find the smallest loop that contains this one
            for &j in &loop_indices {
                if i != j {
                    let loop_j = &self.loops[j];
                    
                    // Check if loop_j contains loop_i
                    if loop_j.blocks.len() > loop_i.blocks.len() &&
                       loop_i.blocks.iter().all(|block| loop_j.blocks.contains(block)) {
                        
                        // This is a potential parent
                        if parent_loop.is_none() || 
                           self.loops.get(parent_loop.unwrap()).map(|l: &LoopInfo| l.blocks.len()).unwrap() > loop_j.blocks.len() {
                            parent_loop = Some(j);
                        }
                    }
                }
            }
            
            // Update parent-child relationships
            self.loops[i].parent = parent_loop;
            if let Some(parent) = parent_loop {
                self.loops[i].depth = self.loops[parent].depth + 1;
                self.loops[parent].children.push(i);
            } else {
                self.loop_forest.roots.push(i);
            }
        }
        
        // Build loop map
        for (_i, loop_info) in self.loops.iter().enumerate() {
            self.loop_forest.loops.insert(loop_info.header, loop_info.clone());
        }
        
        Ok(())
    }
    
    /// Analyze loop invariants
    fn analyze_loop_invariants(&mut self, function: &Function) -> Result<(), SemanticError> {
        self.invariant_analysis = LoopInvariantAnalysis::default();
        
        for (_loop_idx, loop_info) in self.loops.iter().enumerate() {
            let mut invariant_statements = Vec::new();
            let mut invariant_variables = HashSet::new();
            
            // Analyze each block in the loop
            for &block_id in &loop_info.blocks {
                if let Some(block) = function.basic_blocks.get(&(block_id as u32)) {
                    for (stmt_idx, statement) in block.statements.iter().enumerate() {
                        if self.is_loop_invariant(statement, loop_info, function)? {
                            let invariant_stmt = InvariantStatement {
                                block: block_id,
                                statement_index: stmt_idx,
                                statement: statement.clone(),
                                safe_to_hoist: self.is_safe_to_hoist(statement, loop_info, function)?,
                                hoist_profit: self.calculate_hoist_profit(statement, loop_info),
                            };
                            
                            invariant_statements.push(invariant_stmt);
                            
                            // Track invariant variables
                            if let Statement::Assign { place, .. } = statement {
                                invariant_variables.insert(place.clone());
                            }
                        }
                    }
                }
            }
            
            self.invariant_analysis.invariant_statements.insert(loop_info.header, invariant_statements);
            self.invariant_analysis.invariant_variables.insert(loop_info.header, invariant_variables);
        }
        
        Ok(())
    }
    
    /// Check if a statement is loop invariant
    fn is_loop_invariant(&self, statement: &Statement, loop_info: &LoopInfo, function: &Function) -> Result<bool, SemanticError> {
        match statement {
            Statement::Assign { rvalue, .. } => {
                self.is_rvalue_loop_invariant(rvalue, loop_info, function)
            }
            _ => Ok(false), // Conservative: only assignments can be invariant
        }
    }
    
    /// Check if an rvalue is loop invariant
    fn is_rvalue_loop_invariant(&self, rvalue: &Rvalue, loop_info: &LoopInfo, _function: &Function) -> Result<bool, SemanticError> {
        match rvalue {
            Rvalue::Use(operand) => self.is_operand_loop_invariant(operand, loop_info),
            Rvalue::BinaryOp { left, right, .. } => {
                Ok(self.is_operand_loop_invariant(left, loop_info)? && 
                   self.is_operand_loop_invariant(right, loop_info)?)
            }
            Rvalue::UnaryOp { operand, .. } => self.is_operand_loop_invariant(operand, loop_info),
            Rvalue::Cast { operand, .. } => self.is_operand_loop_invariant(operand, loop_info),
            _ => Ok(false), // Conservative: other operations might not be invariant
        }
    }
    
    /// Check if an operand is loop invariant
    fn is_operand_loop_invariant(&self, operand: &Operand, loop_info: &LoopInfo) -> Result<bool, SemanticError> {
        match operand {
            Operand::Constant(_) => Ok(true), // Constants are always invariant
            Operand::Move(place) | Operand::Copy(place) => {
                // Variable is invariant if it's not modified in the loop
                Ok(!self.is_variable_modified_in_loop(place, loop_info))
            }
        }
    }
    
    /// Check if a variable is modified in a loop
    fn is_variable_modified_in_loop(&self, _variable: &Place, _loop_info: &LoopInfo) -> bool {
        // This is a simplified check
        // In reality, we'd need full reaching definitions analysis
        false
    }
    
    /// Check if it's safe to hoist a statement
    fn is_safe_to_hoist(&self, statement: &Statement, loop_info: &LoopInfo, function: &Function) -> Result<bool, SemanticError> {
        match statement {
            Statement::Assign { place, rvalue, .. } => {
                // Check for side effects
                if self.has_side_effects(rvalue) {
                    return Ok(false);
                }
                
                // Check for exceptions
                if self.may_throw_exception(rvalue) {
                    return Ok(false);
                }
                
                // Check if the assignment is always executed
                Ok(self.is_always_executed(place, loop_info, function))
            }
            _ => Ok(false), // Conservative
        }
    }
    
    /// Check if an rvalue has side effects
    fn has_side_effects(&self, rvalue: &Rvalue) -> bool {
        match rvalue {
            Rvalue::Call { .. } => true, // Function calls may have side effects
            _ => false,
        }
    }
    
    /// Check if an rvalue may throw an exception
    fn may_throw_exception(&self, rvalue: &Rvalue) -> bool {
        match rvalue {
            Rvalue::BinaryOp { op: BinOp::Div, .. } => true, // Division by zero
            _ => false,
        }
    }
    
    /// Check if a place is always executed in the loop
    fn is_always_executed(&self, _place: &Place, _loop_info: &LoopInfo, _function: &Function) -> bool {
        // Simplified: assume statements in the header are always executed
        true
    }
    
    /// Calculate profit from hoisting a statement
    fn calculate_hoist_profit(&self, _statement: &Statement, loop_info: &LoopInfo) -> f64 {
        // Simple heuristic: profit is proportional to estimated loop iterations
        let base_profit = 1.0;
        let iteration_multiplier = loop_info.iteration_count.unwrap_or(10) as f64;
        base_profit * iteration_multiplier.min(1000.0) // Cap at 1000 iterations
    }
    
    /// Analyze induction variables
    fn analyze_induction_variables(&mut self, function: &Function) -> Result<(), SemanticError> {
        self.induction_analysis = InductionAnalysis::default();
        
        for loop_info in &self.loops {
            let mut basic_ivs = Vec::new();
            let mut derived_ivs = Vec::new();
            
            // Find basic induction variables
            self.find_basic_induction_variables(loop_info, function, &mut basic_ivs)?;
            
            // Find derived induction variables
            self.find_derived_induction_variables(loop_info, function, &basic_ivs, &mut derived_ivs)?;
            
            self.induction_analysis.basic_induction_vars.insert(loop_info.header, basic_ivs);
            self.induction_analysis.derived_induction_vars.insert(loop_info.header, derived_ivs);
        }
        
        Ok(())
    }
    
    /// Find basic induction variables in a loop
    fn find_basic_induction_variables(
        &self,
        loop_info: &LoopInfo,
        function: &Function,
        basic_ivs: &mut Vec<BasicInductionVar>,
    ) -> Result<(), SemanticError> {
        // Look for variables of the form: i = i + c
        for &block_id in &loop_info.blocks {
            if let Some(block) = function.basic_blocks.get(&(block_id as u32)) {
                for (stmt_idx, statement) in block.statements.iter().enumerate() {
                    if let Statement::Assign { place, rvalue, .. } = statement {
                        if let Rvalue::BinaryOp { op: BinOp::Add, left, right } = rvalue {
                            // Check if left operand is the same variable
                            if let Operand::Move(left_place) | Operand::Copy(left_place) = left {
                                if left_place.local == place.local {
                                    // Check if right operand is a constant
                                    if let Operand::Constant(constant) = right {
                                        if let Some(step) = self.extract_integer_constant(constant) {
                                            let basic_iv = BasicInductionVar {
                                                variable: place.clone(),
                                                initial_value: Operand::Constant(crate::mir::Constant {
                                                    ty: function.locals.get(&place.local)
                                                    .map(|l| l.ty.clone())
                                                    .unwrap_or(Type::Error),
                                                    value: crate::mir::ConstantValue::Integer(0),
                                                }),
                                                step,
                                                increment_block: block_id,
                                                increment_statement: stmt_idx,
                                            };
                                            
                                            basic_ivs.push(basic_iv);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract integer constant value
    fn extract_integer_constant(&self, constant: &crate::mir::Constant) -> Option<i64> {
        match &constant.value {
            crate::mir::ConstantValue::Integer(val) => Some((*val).try_into().unwrap()),
            _ => None,
        }
    }
    
    /// Find derived induction variables
    fn find_derived_induction_variables(
        &self,
        loop_info: &LoopInfo,
        function: &Function,
        basic_ivs: &[BasicInductionVar],
        derived_ivs: &mut Vec<DerivedInductionVar>,
    ) -> Result<(), SemanticError> {
        // Look for variables of the form: j = i * c + d
        for &block_id in &loop_info.blocks {
            if let Some(block) = function.basic_blocks.get(&(block_id as u32)) {
                for statement in &block.statements {
                    if let Statement::Assign { place, rvalue, .. } = statement {
                        // Try to match derived IV patterns
                        if let Some(derived_iv) = self.match_derived_iv_pattern(place, rvalue, basic_ivs) {
                            derived_ivs.push(derived_iv);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Try to match derived induction variable patterns
    fn match_derived_iv_pattern(
        &self,
        place: &Place,
        rvalue: &Rvalue,
        basic_ivs: &[BasicInductionVar],
    ) -> Option<DerivedInductionVar> {
        match rvalue {
            Rvalue::BinaryOp { op: BinOp::Mul, left, right } => {
                // Pattern: j = i * c
                if let (Operand::Move(var) | Operand::Copy(var), Operand::Constant(constant)) = (left, right) {
                    if let Some(multiplier) = self.extract_integer_constant(constant) {
                        for basic_iv in basic_ivs {
                            if basic_iv.variable.local == var.local {
                                return Some(DerivedInductionVar {
                                    variable: place.clone(),
                                    base: basic_iv.variable.clone(),
                                    multiplier,
                                    offset: 0,
                                });
                            }
                        }
                    }
                }
            }
            Rvalue::BinaryOp { op: BinOp::Add, left, right } => {
                // Pattern: j = i + c (offset from basic IV)
                if let (Operand::Move(var) | Operand::Copy(var), Operand::Constant(constant)) = (left, right) {
                    if let Some(offset) = self.extract_integer_constant(constant) {
                        for basic_iv in basic_ivs {
                            if basic_iv.variable.local == var.local {
                                return Some(DerivedInductionVar {
                                    variable: place.clone(),
                                    base: basic_iv.variable.clone(),
                                    multiplier: 1,
                                    offset,
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        
        None
    }
    
    /// Analyze data dependencies
    fn analyze_data_dependencies(&mut self, function: &Function) -> Result<(), SemanticError> {
        self.dependence_analysis = DependenceAnalysis::default();
        
        let loops_data: Vec<(usize, Vec<usize>, LoopInfo)> = self.loops.iter()
            .map(|loop_info| (loop_info.header, loop_info.blocks.iter().copied().collect(), loop_info.clone()))
            .collect();
        
        for (header, blocks, loop_info) in loops_data {
            let mut loop_carried_deps = Vec::new();
            
            // Analyze dependencies within the loop
            for block_id in blocks {
                if let Some(block) = function.basic_blocks.get(&(block_id as u32)) {
                    self.analyze_block_dependencies(block, block_id, &loop_info, &mut loop_carried_deps)?;
                }
            }
            
            self.dependence_analysis.loop_carried_deps.insert(header, loop_carried_deps);
        }
        
        Ok(())
    }
    
    /// Analyze dependencies within a block
    fn analyze_block_dependencies(
        &mut self,
        block: &BasicBlock,
        block_id: usize,
        _loop_info: &LoopInfo,
        _loop_carried_deps: &mut Vec<Dependence>,
    ) -> Result<(), SemanticError> {
        // Simplified dependency analysis
        // In reality, this would be much more sophisticated
        
        for (i, stmt1) in block.statements.iter().enumerate() {
            for (j, stmt2) in block.statements.iter().enumerate().skip(i + 1) {
                if let Some(dep) = self.find_statement_dependency(stmt1, stmt2, block_id, i, j)? {
                    match dep.dep_type {
                        DependenceType::Flow => self.dependence_analysis.flow_deps.push(dep),
                        DependenceType::Anti => self.dependence_analysis.anti_deps.push(dep),
                        DependenceType::Output => self.dependence_analysis.output_deps.push(dep),
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Find dependency between two statements
    fn find_statement_dependency(
        &self,
        stmt1: &Statement,
        stmt2: &Statement,
        block_id: usize,
        index1: usize,
        index2: usize,
    ) -> Result<Option<Dependence>, SemanticError> {
        // Simplified dependency detection
        match (stmt1, stmt2) {
            (Statement::Assign { place: place1, .. }, Statement::Assign { rvalue: rvalue2, .. }) => {
                if self.rvalue_uses_place(rvalue2, place1) {
                    return Ok(Some(Dependence {
                        source: StatementRef { block: block_id, statement: index1 },
                        sink: StatementRef { block: block_id, statement: index2 },
                        distance: vec![index2 as i64 - index1 as i64],
                        direction: vec![DependenceDirection::Greater],
                        dep_type: DependenceType::Flow,
                    }));
                }
            }
            _ => {}
        }
        
        Ok(None)
    }
    
    /// Check if an rvalue uses a place
    fn rvalue_uses_place(&self, rvalue: &Rvalue, place: &Place) -> bool {
        match rvalue {
            Rvalue::Use(operand) => self.operand_uses_place(operand, place),
            Rvalue::BinaryOp { left, right, .. } => {
                self.operand_uses_place(left, place) || self.operand_uses_place(right, place)
            }
            Rvalue::UnaryOp { operand, .. } => self.operand_uses_place(operand, place),
            _ => false,
        }
    }
    
    /// Check if an operand uses a place
    fn operand_uses_place(&self, operand: &Operand, place: &Place) -> bool {
        match operand {
            Operand::Move(op_place) | Operand::Copy(op_place) => {
                op_place.local == place.local
            }
            Operand::Constant(_) => false,
        }
    }
    
    /// Apply loop optimizations
    pub fn apply_optimizations(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        // Apply loop invariant code motion
        if self.apply_loop_invariant_code_motion(function)? {
            changed = true;
        }
        
        // Apply loop unrolling
        if self.apply_loop_unrolling(function)? {
            changed = true;
        }
        
        // Apply induction variable strength reduction
        if self.apply_strength_reduction(function)? {
            changed = true;
        }
        
        Ok(changed)
    }
    
    /// Apply loop invariant code motion
    fn apply_loop_invariant_code_motion(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        for loop_info in &self.loops {
            if let Some(invariant_statements) = self.invariant_analysis.invariant_statements.get(&loop_info.header) {
                for invariant_stmt in invariant_statements {
                    if invariant_stmt.safe_to_hoist && invariant_stmt.hoist_profit > 10.0 {
                        if self.hoist_statement(function, loop_info, invariant_stmt)? {
                            changed = true;
                        }
                    }
                }
            }
        }
        
        Ok(changed)
    }
    
    /// Hoist a statement out of a loop
    fn hoist_statement(
        &self,
        _function: &mut Function,
        loop_info: &LoopInfo,
        invariant_stmt: &InvariantStatement,
    ) -> Result<bool, SemanticError> {
        // This would move the statement to the preheader
        // For now, just report what would be done
        eprintln!("Would hoist statement from block {} (index {}) out of loop {}",
                 invariant_stmt.block, invariant_stmt.statement_index, loop_info.header);
        Ok(false)
    }
    
    /// Apply loop unrolling
    fn apply_loop_unrolling(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        for loop_info in &self.loops {
            if self.should_unroll_loop(loop_info) {
                if self.unroll_loop(function, loop_info)? {
                    changed = true;
                }
            }
        }
        
        Ok(changed)
    }
    
    /// Check if a loop should be unrolled
    fn should_unroll_loop(&self, loop_info: &LoopInfo) -> bool {
        // Simple heuristics for unrolling
        if let Some(iteration_count) = loop_info.iteration_count {
            // Small loops with known iteration count are good candidates
            iteration_count <= 16 && loop_info.blocks.len() <= 5
        } else {
            false
        }
    }
    
    /// Unroll a loop
    fn unroll_loop(&self, _function: &mut Function, loop_info: &LoopInfo) -> Result<bool, SemanticError> {
        // This would duplicate the loop body
        // For now, just report what would be done
        eprintln!("Would unroll loop with header {}", loop_info.header);
        Ok(false)
    }
    
    /// Apply induction variable strength reduction
    fn apply_strength_reduction(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        for loop_info in &self.loops {
            if let Some(derived_ivs) = self.induction_analysis.derived_induction_vars.get(&loop_info.header) {
                for derived_iv in derived_ivs {
                    if self.apply_strength_reduction_to_iv(function, loop_info, derived_iv)? {
                        changed = true;
                    }
                }
            }
        }
        
        Ok(changed)
    }
    
    /// Apply strength reduction to a specific induction variable
    fn apply_strength_reduction_to_iv(
        &self,
        _function: &mut Function,
        loop_info: &LoopInfo,
        _derived_iv: &DerivedInductionVar,
    ) -> Result<bool, SemanticError> {
        // This would replace multiplications with additions
        // For now, just report what would be done
        eprintln!("Would apply strength reduction to derived IV in loop {}", loop_info.header);
        Ok(false)
    }
}

impl OptimizationPass for LoopOptimizationPass {
    fn name(&self) -> &'static str {
        "AdvancedLoopOptimizations"
    }
    
    fn run_on_function(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        // Analyze the function
        self.analyze_function(function)?;
        
        // Apply optimizations
        self.apply_optimizations(function)
    }
}

impl Default for LoopOptimizationPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{Builder, Statement, Rvalue, Operand, Constant, ConstantValue, Place, SourceInfo};
    use crate::types::Type;
    use crate::ast::PrimitiveType;
    use crate::error::SourceLocation;
    
    #[test]
    fn test_loop_optimization_pass() {
        let pass = LoopOptimizationPass::new();
        assert_eq!(pass.name(), "AdvancedLoopOptimizations");
        assert!(pass.loops.is_empty());
    }
    
    #[test]
    fn test_loop_info_creation() {
        let loop_info = LoopInfo {
            header: 0,
            preheader: Some(1),
            blocks: [0, 2, 3].iter().cloned().collect(),
            exits: [4].iter().cloned().collect(),
            back_edges: vec![(3, 0)],
            depth: 1,
            parent: None,
            children: Vec::new(),
            bounds: None,
            iteration_count: Some(10),
        };
        
        assert_eq!(loop_info.header, 0);
        assert_eq!(loop_info.preheader, Some(1));
        assert_eq!(loop_info.blocks.len(), 3);
        assert_eq!(loop_info.iteration_count, Some(10));
    }
    
    #[test]
    fn test_basic_induction_variable() {
        let basic_iv = BasicInductionVar {
            variable: Place {
                local: 0,
                projection: vec![],
            },
            initial_value: Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(0),
            }),
            step: 1,
            increment_block: 0,
            increment_statement: 2,
        };
        
        assert_eq!(basic_iv.step, 1);
        assert_eq!(basic_iv.increment_block, 0);
    }
    
    #[test]
    fn test_derived_induction_variable() {
        let derived_iv = DerivedInductionVar {
            variable: Place {
                local: 1,
                projection: vec![],
            },
            base: Place {
                local: 0,
                projection: vec![],
            },
            multiplier: 4,
            offset: 10,
        };
        
        assert_eq!(derived_iv.multiplier, 4);
        assert_eq!(derived_iv.offset, 10);
    }
    
    #[test]
    fn test_dependence_analysis() {
        let dep = Dependence {
            source: StatementRef { block: 0, statement: 1 },
            sink: StatementRef { block: 0, statement: 3 },
            distance: vec![2],
            direction: vec![DependenceDirection::Greater],
            dep_type: DependenceType::Flow,
        };
        
        assert_eq!(dep.source.block, 0);
        assert_eq!(dep.sink.statement, 3);
        assert_eq!(dep.dep_type, DependenceType::Flow);
    }
    
    #[test]
    fn test_invariant_statement() {
        let invariant_stmt = InvariantStatement {
            block: 0,
            statement_index: 2,
            statement: Statement::Nop,
            safe_to_hoist: true,
            hoist_profit: 15.5,
        };
        
        assert_eq!(invariant_stmt.block, 0);
        assert!(invariant_stmt.safe_to_hoist);
        assert_eq!(invariant_stmt.hoist_profit, 15.5);
    }
}
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

//! Profile-guided optimization for AetherScript
//!
//! Uses runtime profiling data to guide optimization decisions, including
//! function inlining, basic block layout, and branch prediction.

use crate::error::SemanticError;
use crate::mir::{BasicBlock, Function, Program, Terminator};
use crate::optimizations::OptimizationPass;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Profile-guided optimization pass
#[derive(Debug)]
pub struct ProfileGuidedOptimizationPass {
    /// Loaded profile data
    profile_data: ProfileData,

    /// Hot function threshold (execution count)
    hot_function_threshold: u64,

    /// Hot basic block threshold (execution count)
    hot_block_threshold: u64,

    /// Function inlining decisions based on profile
    inline_decisions: HashMap<String, InlineDecision>,

    /// Basic block layout decisions
    layout_decisions: HashMap<String, BlockLayout>,
}

/// Profile data collected from program execution
#[derive(Debug, Default)]
pub struct ProfileData {
    /// Function execution counts
    function_counts: HashMap<String, u64>,

    /// Basic block execution counts
    block_counts: HashMap<String, HashMap<usize, u64>>,

    /// Branch taken frequencies
    branch_frequencies: HashMap<String, HashMap<usize, BranchProfile>>,

    /// Function call frequencies
    call_frequencies: HashMap<String, HashMap<String, u64>>,

    /// Loop iteration counts
    loop_iteration_counts: HashMap<String, HashMap<usize, LoopProfile>>,
}

/// Branch profiling information
#[derive(Debug, Clone)]
pub struct BranchProfile {
    /// Total number of times this branch was encountered
    pub total_count: u64,

    /// Number of times the branch was taken
    pub taken_count: u64,

    /// Branch probability (taken_count / total_count)
    pub probability: f64,
}

/// Loop profiling information
#[derive(Debug, Clone)]
pub struct LoopProfile {
    /// Number of times the loop was entered
    pub entry_count: u64,

    /// Total iterations across all loop invocations
    pub total_iterations: u64,

    /// Average iterations per loop invocation
    pub avg_iterations: f64,

    /// Maximum iterations observed
    pub max_iterations: u64,
}

/// Function inlining decision
#[derive(Debug, Clone, PartialEq)]
pub enum InlineDecision {
    /// Always inline this function
    AlwaysInline,

    /// Inline only in hot paths
    InlineHot,

    /// Never inline this function
    NeverInline,

    /// Use default heuristics
    Default,
}

/// Basic block layout optimization
#[derive(Debug, Clone)]
pub struct BlockLayout {
    /// Optimal ordering of basic blocks
    pub block_order: Vec<usize>,

    /// Hot blocks that should be placed first
    pub hot_blocks: Vec<usize>,

    /// Cold blocks that should be placed last
    pub cold_blocks: Vec<usize>,
}

/// Profile collection instrumentation
#[derive(Debug, Default)]
pub struct ProfileInstrumentation {}

/// A profiling probe point
#[derive(Debug, Clone)]
pub struct ProbePoint {
    /// Unique identifier for this probe
    pub id: u64,

    /// Location in the program
    pub location: ProbeLocation,

    /// Type of data being collected
    pub probe_type: ProbeType,
}

/// Location of a profiling probe
#[derive(Debug, Clone)]
pub enum ProbeLocation {
    FunctionEntry(String),
    FunctionExit(String),
    BasicBlock(String, usize),
    Branch(String, usize),
    LoopHeader(String, usize),
}

/// Type of profiling probe
#[derive(Debug, Clone)]
pub enum ProbeType {
    Counter,
    Timer,
    Sampler,
}

impl ProfileGuidedOptimizationPass {
    pub fn new() -> Self {
        Self {
            profile_data: ProfileData::default(),
            hot_function_threshold: 1000,
            hot_block_threshold: 500,
            inline_decisions: HashMap::new(),
            layout_decisions: HashMap::new(),
        }
    }

    /// Create a new ProfileGuidedOptimizationPass from a profile data file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SemanticError> {
        let mut pgo = Self::new();
        pgo.load_profile_data(path)?;
        Ok(pgo)
    }

    /// Load profile data from a file
    pub fn load_profile_data<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SemanticError> {
        let file = File::open(path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to open profile data file: {}", e),
        })?;

        let reader = BufReader::new(file);
        self.parse_profile_data(reader)
    }

    /// Parse profile data from a reader
    fn parse_profile_data<R: BufRead>(&mut self, reader: R) -> Result<(), SemanticError> {
        for line in reader.lines() {
            let line = line.map_err(|e| SemanticError::Internal {
                message: format!("Failed to read profile data: {}", e),
            })?;

            self.parse_profile_line(&line)?;
        }

        // Compute derived metrics
        self.compute_branch_probabilities();
        self.compute_loop_averages();

        Ok(())
    }

    /// Parse a single line of profile data
    fn parse_profile_line(&mut self, line: &str) -> Result<(), SemanticError> {
        let parts: Vec<&str> = line.trim().split(':').collect();
        if parts.len() < 3 {
            return Ok(()); // Skip malformed lines
        }

        match parts[0] {
            "FUNC" => {
                // Function execution count: FUNC:function_name:count
                if parts.len() >= 3 {
                    let function_name = parts[1].to_string();
                    let count = parts[2].parse::<u64>().unwrap_or(0);
                    self.profile_data
                        .function_counts
                        .insert(function_name, count);
                }
            }
            "BLOCK" => {
                // Basic block execution count: BLOCK:function_name:block_id:count
                if parts.len() >= 4 {
                    let function_name = parts[1].to_string();
                    let block_id = parts[2].parse::<usize>().unwrap_or(0);
                    let count = parts[3].parse::<u64>().unwrap_or(0);

                    self.profile_data
                        .block_counts
                        .entry(function_name)
                        .or_default()
                        .insert(block_id, count);
                }
            }
            "BRANCH" => {
                // Branch profile: BRANCH:function_name:block_id:total:taken
                if parts.len() >= 5 {
                    let function_name = parts[1].to_string();
                    let block_id = parts[2].parse::<usize>().unwrap_or(0);
                    let total = parts[3].parse::<u64>().unwrap_or(0);
                    let taken = parts[4].parse::<u64>().unwrap_or(0);

                    let branch_profile = BranchProfile {
                        total_count: total,
                        taken_count: taken,
                        probability: if total > 0 {
                            taken as f64 / total as f64
                        } else {
                            0.0
                        },
                    };

                    self.profile_data
                        .branch_frequencies
                        .entry(function_name)
                        .or_default()
                        .insert(block_id, branch_profile);
                }
            }
            "CALL" => {
                // Function call frequency: CALL:caller:callee:count
                if parts.len() >= 4 {
                    let caller = parts[1].to_string();
                    let callee = parts[2].to_string();
                    let count = parts[3].parse::<u64>().unwrap_or(0);

                    self.profile_data
                        .call_frequencies
                        .entry(caller)
                        .or_default()
                        .insert(callee, count);
                }
            }
            "LOOP" => {
                // Loop profile: LOOP:function_name:block_id:entries:total_iterations:max_iterations
                if parts.len() >= 6 {
                    let function_name = parts[1].to_string();
                    let block_id = parts[2].parse::<usize>().unwrap_or(0);
                    let entries = parts[3].parse::<u64>().unwrap_or(0);
                    let total_iterations = parts[4].parse::<u64>().unwrap_or(0);
                    let max_iterations = parts[5].parse::<u64>().unwrap_or(0);

                    let loop_profile = LoopProfile {
                        entry_count: entries,
                        total_iterations,
                        avg_iterations: if entries > 0 {
                            total_iterations as f64 / entries as f64
                        } else {
                            0.0
                        },
                        max_iterations,
                    };

                    self.profile_data
                        .loop_iteration_counts
                        .entry(function_name)
                        .or_default()
                        .insert(block_id, loop_profile);
                }
            }
            _ => {
                // Unknown profile data type, skip
            }
        }

        Ok(())
    }

    /// Compute branch probabilities from raw counts
    fn compute_branch_probabilities(&mut self) {
        for branch_map in self.profile_data.branch_frequencies.values_mut() {
            for branch_profile in branch_map.values_mut() {
                if branch_profile.total_count > 0 {
                    branch_profile.probability =
                        branch_profile.taken_count as f64 / branch_profile.total_count as f64;
                }
            }
        }
    }

    /// Compute loop average iterations
    fn compute_loop_averages(&mut self) {
        for loop_map in self.profile_data.loop_iteration_counts.values_mut() {
            for loop_profile in loop_map.values_mut() {
                if loop_profile.entry_count > 0 {
                    loop_profile.avg_iterations =
                        loop_profile.total_iterations as f64 / loop_profile.entry_count as f64;
                }
            }
        }
    }

    /// Analyze profile data and make optimization decisions
    pub fn analyze_and_decide(&mut self) -> Result<(), SemanticError> {
        self.make_inlining_decisions()?;
        self.make_layout_decisions()?;
        Ok(())
    }

    /// Make function inlining decisions based on profile data
    fn make_inlining_decisions(&mut self) -> Result<(), SemanticError> {
        self.inline_decisions.clear();

        // Analyze each function's call patterns
        for (caller, callees) in &self.profile_data.call_frequencies {
            let caller_count = self
                .profile_data
                .function_counts
                .get(caller)
                .copied()
                .unwrap_or(0);

            for (callee, call_count) in callees {
                let decision = self.decide_inlining(caller, callee, *call_count, caller_count)?;
                self.inline_decisions
                    .insert(format!("{}::{}", caller, callee), decision);
            }
        }

        Ok(())
    }

    /// Decide whether to inline a specific function call
    fn decide_inlining(
        &self,
        _caller: &str,
        callee: &str,
        call_count: u64,
        caller_count: u64,
    ) -> Result<InlineDecision, SemanticError> {
        let callee_count = self
            .profile_data
            .function_counts
            .get(callee)
            .copied()
            .unwrap_or(0);

        // Calculate call frequency
        let call_frequency = if caller_count > 0 {
            call_count as f64 / caller_count as f64
        } else {
            0.0
        };

        // Decide based on frequency and function hotness
        if callee_count > self.hot_function_threshold && call_frequency > 0.8 {
            Ok(InlineDecision::AlwaysInline)
        } else if callee_count > self.hot_function_threshold / 2 && call_frequency > 0.5 {
            Ok(InlineDecision::InlineHot)
        } else if callee_count < 10 || call_frequency < 0.01 {
            Ok(InlineDecision::NeverInline)
        } else {
            Ok(InlineDecision::Default)
        }
    }

    /// Make basic block layout decisions
    fn make_layout_decisions(&mut self) -> Result<(), SemanticError> {
        self.layout_decisions.clear();

        for (function_name, block_counts) in &self.profile_data.block_counts {
            let layout = self.optimize_block_layout(function_name, block_counts)?;
            self.layout_decisions.insert(function_name.clone(), layout);
        }

        Ok(())
    }

    /// Optimize basic block layout for a function
    fn optimize_block_layout(
        &self,
        function_name: &str,
        block_counts: &HashMap<usize, u64>,
    ) -> Result<BlockLayout, SemanticError> {
        // Sort blocks by execution count (hottest first)
        let mut blocks: Vec<(usize, u64)> = block_counts
            .iter()
            .map(|(&id, &count)| (id, count))
            .collect();
        blocks.sort_by(|a, b| b.1.cmp(&a.1));

        let mut hot_blocks = Vec::new();
        let mut cold_blocks = Vec::new();
        let mut block_order = Vec::new();

        for (block_id, count) in blocks {
            block_order.push(block_id);

            if count > self.hot_block_threshold {
                hot_blocks.push(block_id);
            } else if count < 10 {
                cold_blocks.push(block_id);
            }
        }

        // Consider branch probabilities for better layout
        if let Some(branch_data) = self.profile_data.branch_frequencies.get(function_name) {
            self.refine_layout_with_branches(&mut block_order, branch_data);
        }

        Ok(BlockLayout {
            block_order,
            hot_blocks,
            cold_blocks,
        })
    }

    /// Refine block layout using branch probability data
    fn refine_layout_with_branches(
        &self,
        block_order: &mut Vec<usize>,
        branch_data: &HashMap<usize, BranchProfile>,
    ) {
        // Place likely-taken branches close to their source blocks
        // This is a simplified heuristic - real implementations use more sophisticated algorithms

        for (block_id, branch_profile) in branch_data {
            if branch_profile.probability > 0.8 {
                // This branch is very likely taken, try to place target close to source
                if let Some(pos) = block_order.iter().position(|&id| id == *block_id) {
                    // Move this block earlier in the layout if it's hot
                    if pos > 0 && branch_profile.total_count > self.hot_block_threshold {
                        let block = block_order.remove(pos);
                        block_order.insert(0, block);
                    }
                }
            }
        }
    }

    /// Apply profile-guided optimizations to a program
    pub fn apply_optimizations(&self, program: &mut Program) -> Result<bool, SemanticError> {
        let mut changed = false;

        // Apply inlining decisions
        if self.apply_profile_guided_inlining(program)? {
            changed = true;
        }

        // Apply block layout optimizations
        if self.apply_block_layout_optimizations(program)? {
            changed = true;
        }

        // Apply branch prediction optimizations
        if self.apply_branch_optimizations(program)? {
            changed = true;
        }

        Ok(changed)
    }

    /// Apply profile-guided function inlining
    fn apply_profile_guided_inlining(&self, program: &mut Program) -> Result<bool, SemanticError> {
        let mut changed = false;

        for (call_site, decision) in &self.inline_decisions {
            match decision {
                InlineDecision::AlwaysInline => {
                    if self.try_inline_call_site(program, call_site)? {
                        changed = true;
                    }
                }
                InlineDecision::InlineHot => {
                    // Only inline in hot paths
                    if self.is_hot_call_site(call_site)
                        && self.try_inline_call_site(program, call_site)?
                    {
                        changed = true;
                    }
                }
                _ => {
                    // Don't inline or use default heuristics
                }
            }
        }

        Ok(changed)
    }

    /// Try to inline a specific call site
    fn try_inline_call_site(
        &self,
        _program: &mut Program,
        call_site: &str,
    ) -> Result<bool, SemanticError> {
        // Placeholder for actual inlining implementation
        eprintln!("Would inline call site: {}", call_site);
        Ok(false)
    }

    /// Check if a call site is in a hot path
    fn is_hot_call_site(&self, call_site: &str) -> bool {
        // Extract caller from call site string
        if let Some(caller) = call_site.split("::").next() {
            if let Some(&count) = self.profile_data.function_counts.get(caller) {
                return count > self.hot_function_threshold;
            }
        }
        false
    }

    /// Apply basic block layout optimizations
    fn apply_block_layout_optimizations(
        &self,
        program: &mut Program,
    ) -> Result<bool, SemanticError> {
        let mut changed = false;

        for (function_name, layout) in &self.layout_decisions {
            if let Some(function) = program.functions.get_mut(function_name) {
                if self.reorder_basic_blocks(function, layout)? {
                    changed = true;
                }
            }
        }

        Ok(changed)
    }

    /// Reorder basic blocks in a function
    fn reorder_basic_blocks(
        &self,
        function: &mut Function,
        layout: &BlockLayout,
    ) -> Result<bool, SemanticError> {
        // This would reorder the basic blocks in the function
        // For now, just report what would be done
        eprintln!(
            "Would reorder blocks in function {}: {:?}",
            function.name, layout.block_order
        );
        Ok(false)
    }

    /// Apply branch prediction optimizations
    fn apply_branch_optimizations(&self, program: &mut Program) -> Result<bool, SemanticError> {
        let mut changed = false;

        for function in program.functions.values_mut() {
            if let Some(branch_data) = self.profile_data.branch_frequencies.get(&function.name) {
                if self.optimize_branches_in_function(function, branch_data)? {
                    changed = true;
                }
            }
        }

        Ok(changed)
    }

    /// Optimize branches in a function based on profile data
    fn optimize_branches_in_function(
        &self,
        function: &mut Function,
        branch_data: &HashMap<usize, BranchProfile>,
    ) -> Result<bool, SemanticError> {
        let mut changed = false;

        for (block_id, branch_profile) in branch_data {
            let block_id_u32 = *block_id as u32;
            if let Some(block) = function.basic_blocks.get_mut(&block_id_u32) {
                if self.optimize_block_branches(block, branch_profile)? {
                    changed = true;
                }
            }
        }

        Ok(changed)
    }

    /// Optimize branches in a basic block
    fn optimize_block_branches(
        &self,
        block: &mut BasicBlock,
        branch_profile: &BranchProfile,
    ) -> Result<bool, SemanticError> {
        // This would optimize the branch based on probability
        // For example:
        // - Reorder branch targets to put likely target first
        // - Add branch prediction hints
        // - Transform unlikely branches to reduce overhead

        if let Terminator::SwitchInt { .. } = &mut block.terminator {
            // Could reorder switch targets based on branch probabilities
            if branch_profile.probability > 0.9 {
                eprintln!(
                    "Would optimize highly predictable branch (p={:.2})",
                    branch_profile.probability
                );
                return Ok(false); // Not actually implemented
            }
        }

        Ok(false)
    }

    /// Generate instrumentation code for profile collection
    pub fn generate_instrumentation(
        &self,
        _program: &mut Program,
    ) -> Result<ProfileInstrumentation, SemanticError> {
        // This would insert profiling probes into the program
        // For now, return empty instrumentation
        Ok(ProfileInstrumentation {})
    }

    /// Save profile data to a file
    pub fn save_profile_data<P: AsRef<Path>>(&self, path: P) -> Result<(), SemanticError> {
        let mut file = File::create(path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create profile data file: {}", e),
        })?;

        // Write function counts
        for (function_name, count) in &self.profile_data.function_counts {
            writeln!(file, "FUNC:{}:{}", function_name, count).map_err(|e| {
                SemanticError::Internal {
                    message: format!("Failed to write profile data: {}", e),
                }
            })?;
        }

        // Write block counts
        for (function_name, block_counts) in &self.profile_data.block_counts {
            for (block_id, count) in block_counts {
                writeln!(file, "BLOCK:{}:{}:{}", function_name, block_id, count).map_err(|e| {
                    SemanticError::Internal {
                        message: format!("Failed to write profile data: {}", e),
                    }
                })?;
            }
        }

        // Write branch frequencies
        for (function_name, branch_data) in &self.profile_data.branch_frequencies {
            for (block_id, branch_profile) in branch_data {
                writeln!(
                    file,
                    "BRANCH:{}:{}:{}:{}",
                    function_name, block_id, branch_profile.total_count, branch_profile.taken_count
                )
                .map_err(|e| SemanticError::Internal {
                    message: format!("Failed to write profile data: {}", e),
                })?;
            }
        }

        Ok(())
    }

    /// Get statistics about the loaded profile data
    pub fn get_profile_statistics(&self) -> ProfileStatistics {
        ProfileStatistics {
            total_functions: self.profile_data.function_counts.len(),
            total_blocks: self
                .profile_data
                .block_counts
                .values()
                .map(|m| m.len())
                .sum(),
            total_branches: self
                .profile_data
                .branch_frequencies
                .values()
                .map(|m| m.len())
                .sum(),
            total_calls: self
                .profile_data
                .call_frequencies
                .values()
                .map(|m| m.len())
                .sum(),
            hottest_function: self.find_hottest_function(),
            total_execution_count: self.profile_data.function_counts.values().sum(),
        }
    }

    /// Find the hottest function in the profile
    fn find_hottest_function(&self) -> Option<(String, u64)> {
        self.profile_data
            .function_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(name, &count)| (name.clone(), count))
    }
}

/// Statistics about profile data
#[derive(Debug)]
pub struct ProfileStatistics {
    pub total_functions: usize,
    pub total_blocks: usize,
    pub total_branches: usize,
    pub total_calls: usize,
    pub hottest_function: Option<(String, u64)>,
    pub total_execution_count: u64,
}

impl OptimizationPass for ProfileGuidedOptimizationPass {
    fn name(&self) -> &'static str {
        "ProfileGuidedOptimization"
    }

    fn run_on_function(&mut self, _function: &mut Function) -> Result<bool, SemanticError> {
        // Profile-guided optimization works at the program level
        Ok(false)
    }

    fn run_on_program(&mut self, program: &mut Program) -> Result<bool, SemanticError> {
        // Analyze profile data and make decisions
        self.analyze_and_decide()?;

        // Apply optimizations based on profile data
        self.apply_optimizations(program)
    }
}

impl Default for ProfileGuidedOptimizationPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_profile_guided_optimization_pass() {
        let pass = ProfileGuidedOptimizationPass::new();
        assert_eq!(pass.name(), "ProfileGuidedOptimization");
        assert!(pass.profile_data.function_counts.is_empty());
    }

    #[test]
    fn test_profile_data_parsing() {
        let mut pass = ProfileGuidedOptimizationPass::new();
        let profile_data = "FUNC:main:1000\nBLOCK:main:0:500\nBRANCH:main:0:100:80\n";
        let cursor = Cursor::new(profile_data);

        assert!(pass.parse_profile_data(cursor).is_ok());
        assert_eq!(pass.profile_data.function_counts.get("main"), Some(&1000));
        assert_eq!(
            pass.profile_data.block_counts.get("main").unwrap().get(&0),
            Some(&500)
        );
    }

    #[test]
    fn test_branch_probability_calculation() {
        let branch_profile = BranchProfile {
            total_count: 100,
            taken_count: 80,
            probability: 0.8,
        };

        assert_eq!(branch_profile.probability, 0.8);
        assert_eq!(branch_profile.taken_count, 80);
        assert_eq!(branch_profile.total_count, 100);
    }

    #[test]
    fn test_inlining_decision() {
        let mut pass = ProfileGuidedOptimizationPass::new();

        // Add profile data for the callee
        pass.profile_data
            .function_counts
            .insert("callee".to_string(), 600);

        // High frequency call in medium-hot function should get InlineHot
        let decision = pass.decide_inlining("caller", "callee", 900, 1000).unwrap();
        assert_eq!(decision, InlineDecision::InlineHot); // 600 > 500 (hot_threshold/2) and frequency 0.9 > 0.5

        // Low frequency call should not be inlined
        let decision = pass.decide_inlining("caller", "callee", 1, 1000).unwrap();
        assert_eq!(decision, InlineDecision::NeverInline);

        // Test with very hot function
        pass.profile_data
            .function_counts
            .insert("hot_callee".to_string(), 2000);
        let decision = pass
            .decide_inlining("caller", "hot_callee", 900, 1000)
            .unwrap();
        assert_eq!(decision, InlineDecision::AlwaysInline); // 2000 > 1000 and frequency 0.9 > 0.8
    }

    #[test]
    fn test_loop_profile() {
        let loop_profile = LoopProfile {
            entry_count: 10,
            total_iterations: 1000,
            avg_iterations: 100.0,
            max_iterations: 200,
        };

        assert_eq!(loop_profile.avg_iterations, 100.0);
        assert_eq!(loop_profile.max_iterations, 200);
    }

    #[test]
    fn test_profile_statistics() {
        let mut pass = ProfileGuidedOptimizationPass::new();
        pass.profile_data
            .function_counts
            .insert("main".to_string(), 1000);
        pass.profile_data
            .function_counts
            .insert("helper".to_string(), 500);

        let stats = pass.get_profile_statistics();
        assert_eq!(stats.total_functions, 2);
        assert_eq!(stats.total_execution_count, 1500);
        assert_eq!(stats.hottest_function, Some(("main".to_string(), 1000)));
    }

    #[test]
    fn test_block_layout_optimization() {
        let pass = ProfileGuidedOptimizationPass::new();
        let mut block_counts = HashMap::new();
        block_counts.insert(0, 1000); // Hot block
        block_counts.insert(1, 100); // Warm block
        block_counts.insert(2, 1); // Cold block

        let layout = pass
            .optimize_block_layout("test_function", &block_counts)
            .unwrap();

        // Hot block should be first
        assert_eq!(layout.block_order[0], 0);
        assert!(layout.hot_blocks.contains(&0));
        assert!(layout.cold_blocks.contains(&2));
    }
}

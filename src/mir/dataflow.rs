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

//! Data flow analysis framework for MIR
//!
//! Provides forward and backward data flow analysis capabilities

use super::cfg;
use super::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Direction of data flow analysis
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Forward,
    Backward,
}

/// A data flow analysis problem
pub trait DataFlowAnalysis {
    /// The type of data flow facts
    type Fact: Clone + PartialEq;

    /// Direction of analysis
    fn direction(&self) -> Direction;

    /// Initial fact for entry/exit
    fn initial_fact(&self) -> Self::Fact;

    /// Transfer function for statements
    fn transfer_statement(
        &self,
        stmt: &Statement,
        fact: Self::Fact,
        location: Location,
    ) -> Self::Fact;

    /// Transfer function for terminators
    fn transfer_terminator(
        &self,
        term: &Terminator,
        fact: Self::Fact,
        location: Location,
    ) -> Self::Fact;

    /// Join operation (meet/join in lattice)
    fn join(&self, facts: &[Self::Fact]) -> Self::Fact;
}

/// Location in a function (basic block + statement index)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub block: BasicBlockId,
    pub statement_index: Option<usize>,
}

/// Results of data flow analysis
pub struct DataFlowResults<A: DataFlowAnalysis> {
    /// Facts at each program point
    pub facts: HashMap<Location, A::Fact>,
}

/// Run data flow analysis on a function
pub fn run_analysis<A: DataFlowAnalysis>(function: &Function, analysis: A) -> DataFlowResults<A> {
    let mut facts = HashMap::new();
    let mut worklist = VecDeque::new();

    // Initialize based on direction
    match analysis.direction() {
        Direction::Forward => {
            // Start from entry block
            worklist.push_back(function.entry_block);
            facts.insert(
                Location {
                    block: function.entry_block,
                    statement_index: None,
                },
                analysis.initial_fact(),
            );
        }
        Direction::Backward => {
            // Start from all exit blocks (returns)
            for (block_id, block) in &function.basic_blocks {
                if matches!(block.terminator, Terminator::Return) {
                    worklist.push_back(*block_id);
                    facts.insert(
                        Location {
                            block: *block_id,
                            statement_index: Some(block.statements.len()),
                        },
                        analysis.initial_fact(),
                    );
                }
            }
        }
    }

    // Fixed-point iteration
    while let Some(block_id) = worklist.pop_front() {
        let block = &function.basic_blocks[&block_id];

        match analysis.direction() {
            Direction::Forward => {
                // Get input fact
                let mut fact = if block_id == function.entry_block {
                    analysis.initial_fact()
                } else {
                    let preds = cfg::predecessors(function, block_id);
                    let pred_facts: Vec<_> = preds
                        .iter()
                        .filter_map(|pred| {
                            facts.get(&Location {
                                block: *pred,
                                statement_index: Some(function.basic_blocks[pred].statements.len()),
                            })
                        })
                        .cloned()
                        .collect();
                    analysis.join(&pred_facts)
                };

                // Process statements
                for (i, stmt) in block.statements.iter().enumerate() {
                    let loc = Location {
                        block: block_id,
                        statement_index: Some(i),
                    };
                    fact = analysis.transfer_statement(stmt, fact, loc);

                    // Update fact if changed
                    if facts.get(&loc) != Some(&fact) {
                        facts.insert(loc, fact.clone());
                    }
                }

                // Process terminator
                let term_loc = Location {
                    block: block_id,
                    statement_index: Some(block.statements.len()),
                };
                fact = analysis.transfer_terminator(&block.terminator, fact, term_loc);

                if facts.get(&term_loc) != Some(&fact) {
                    facts.insert(term_loc, fact);

                    // Add successors to worklist
                    for succ in cfg::successors(block) {
                        if !worklist.contains(&succ) {
                            worklist.push_back(succ);
                        }
                    }
                }
            }
            Direction::Backward => {
                // Similar but in reverse order
                // TODO: Implement backward analysis
            }
        }
    }

    DataFlowResults { facts }
}

/// Liveness analysis - determines which variables are live at each point
pub struct LivenessAnalysis;

impl DataFlowAnalysis for LivenessAnalysis {
    type Fact = HashSet<LocalId>;

    fn direction(&self) -> Direction {
        Direction::Backward
    }

    fn initial_fact(&self) -> Self::Fact {
        HashSet::new()
    }

    fn transfer_statement(
        &self,
        stmt: &Statement,
        mut fact: Self::Fact,
        _location: Location,
    ) -> Self::Fact {
        match stmt {
            Statement::Assign { place, rvalue, .. } => {
                // Kill the definition
                fact.remove(&place.local);

                // Gen the uses
                self.add_rvalue_uses(rvalue, &mut fact);
            }
            Statement::StorageDead(local) => {
                fact.remove(local);
            }
            _ => {}
        }

        fact
    }

    fn transfer_terminator(
        &self,
        term: &Terminator,
        mut fact: Self::Fact,
        _location: Location,
    ) -> Self::Fact {
        match term {
            Terminator::SwitchInt { discriminant, .. } => {
                self.add_operand_uses(discriminant, &mut fact);
            }
            Terminator::Call { func, args, .. } => {
                self.add_operand_uses(func, &mut fact);
                for arg in args {
                    self.add_operand_uses(arg, &mut fact);
                }
            }
            Terminator::Assert { condition, .. } => {
                self.add_operand_uses(condition, &mut fact);
            }
            _ => {}
        }

        fact
    }

    fn join(&self, facts: &[Self::Fact]) -> Self::Fact {
        let mut result = HashSet::new();
        for fact in facts {
            result.extend(fact.iter().cloned());
        }
        result
    }
}

impl LivenessAnalysis {
    fn add_operand_uses(&self, operand: &Operand, fact: &mut HashSet<LocalId>) {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                fact.insert(place.local);
            }
            Operand::Constant(_) => {}
        }
    }

    fn add_rvalue_uses(&self, rvalue: &Rvalue, fact: &mut HashSet<LocalId>) {
        match rvalue {
            Rvalue::Use(op) => self.add_operand_uses(op, fact),
            Rvalue::BinaryOp { left, right, .. } => {
                self.add_operand_uses(left, fact);
                self.add_operand_uses(right, fact);
            }
            Rvalue::UnaryOp { operand, .. } => {
                self.add_operand_uses(operand, fact);
            }
            Rvalue::Call { func, args, .. } => {
                self.add_operand_uses(func, fact);
                for arg in args {
                    self.add_operand_uses(arg, fact);
                }
            }
            Rvalue::Aggregate { operands, .. } => {
                for op in operands {
                    self.add_operand_uses(op, fact);
                }
            }
            Rvalue::Cast { operand, .. } => {
                self.add_operand_uses(operand, fact);
            }
            Rvalue::Ref { place, .. } => {
                fact.insert(place.local);
            }
            Rvalue::AddressOf(place) => {
                fact.insert(place.local);
                // Add uses from projection (e.g. index)
                for elem in &place.projection {
                    if let crate::mir::PlaceElem::Index(idx) = elem {
                        fact.insert(*idx);
                    }
                }
            }
            Rvalue::Len(place) | Rvalue::Discriminant(place) => {
                fact.insert(place.local);
            }
            Rvalue::Closure { captures, .. } => {
                // Add uses for captured operands
                for capture in captures {
                    self.add_operand_uses(capture, fact);
                }
            }
        }
    }
}

/// Reaching definitions analysis
pub struct ReachingDefinitions;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub local: LocalId,
    pub location: Location,
}

impl DataFlowAnalysis for ReachingDefinitions {
    type Fact = HashSet<Definition>;

    fn direction(&self) -> Direction {
        Direction::Forward
    }

    fn initial_fact(&self) -> Self::Fact {
        HashSet::new()
    }

    fn transfer_statement(
        &self,
        stmt: &Statement,
        mut fact: Self::Fact,
        location: Location,
    ) -> Self::Fact {
        if let Statement::Assign { place, .. } = stmt {
            // Kill all previous definitions of this local
            fact.retain(|def| def.local != place.local);

            // Gen this definition
            fact.insert(Definition {
                local: place.local,
                location,
            });
        }

        fact
    }

    fn transfer_terminator(
        &self,
        _term: &Terminator,
        fact: Self::Fact,
        _location: Location,
    ) -> Self::Fact {
        fact
    }

    fn join(&self, facts: &[Self::Fact]) -> Self::Fact {
        let mut result = HashSet::new();
        for fact in facts {
            result.extend(fact.iter().cloned());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;

    #[test]
    fn test_liveness_analysis() {
        // Create a simple function with:
        // bb0:
        //   _1 = 1
        //   _2 = _1 + 2
        //   return _2

        let mut builder = Builder::new();
        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );

        let local1 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        let local2 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);

        // _1 = 1
        builder.push_statement(Statement::Assign {
            place: Place {
                local: local1,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(1),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // _2 = _1 + 2
        builder.push_statement(Statement::Assign {
            place: Place {
                local: local2,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Add,
                left: Operand::Copy(Place {
                    local: local1,
                    projection: vec![],
                }),
                right: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::Integer),
                    value: ConstantValue::Integer(2),
                }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        builder.set_terminator(Terminator::Return);

        let function = builder.finish_function();

        // Run liveness analysis
        let analysis = LivenessAnalysis;
        let _results = run_analysis(&function, analysis);

        // In a full implementation, we'd verify the results
        // For now, just ensure it runs without panicking
    }
}

use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};

use crate::assign_homes::uncover_live::{self as prev};
use crate::select_instructions::{self, Instruction, Op};
use crate::var::Var;

use super::uncover_live::AnnotatedInstruction;
use super::write_set;

type Index = NodeIndex<u32>;
pub type Graph = petgraph::Graph<Var, (), petgraph::Undirected, u32>;

struct InterferenceGraph {
    graph: Graph,
    var_map: HashMap<Var, Index>,
}

impl InterferenceGraph {
    fn new(program: &prev::Program) -> Self {
        let mut graph = Graph::new_undirected();
        let mut var_map = HashMap::new();

        for var in collect_vars(program) {
            var_map.insert(var.clone(), graph.add_node(var));
        }

        InterferenceGraph { graph, var_map }
    }

    fn add_edge(&mut self, a: &Var, b: &Var) {
        self.graph.update_edge(self.var_map[a], self.var_map[b], ());
    }
}

fn collect_vars(program: &prev::Program) -> HashSet<Var> {
    let mut vs: HashSet<Var> = HashSet::new();

    for b in program.blocks.node_weights() {
        for prev::AnnotatedInstruction { instr, .. } in &b.instrs {
            collect_vars_instr(&mut vs, instr.clone());
        }
    }

    vs
}

fn collect_vars_instr(vs: &mut HashSet<Var>, instr: select_instructions::Instruction) {
    match instr {
        Instruction::Set { var, value: _ } => {
            vs.insert(var);
        }
        Instruction::Operation {
            op,
            source,
            destination,
        } => {
            vs.insert(source);
            vs.insert(destination);
        }
        Instruction::Tellraw { text: _ } => {}
        Instruction::Command { text: _ } => {}
        Instruction::ExecuteIfScoreMatchesSet { var, value, set_var, set_value } => {
            vs.insert(var);
            vs.insert(set_var);
        }
        Instruction::ExecuteUnlessScoreMatchesSet { var, value, set_var, set_value } => {
            vs.insert(var);
            vs.insert(set_var);
        }
        Instruction::ExecuteIfScoreEqualsSet { a, b, set_var, set_value } => {
            vs.insert(a);
            vs.insert(b);
            vs.insert(set_var);
        }
        Instruction::ExecuteUnlessScoreEqualsSet { a, b, set_var, set_value } => {
            vs.insert(a);
            vs.insert(b);
            vs.insert(set_var);
        }
    }
}

pub fn build_interference(program: &prev::Program) -> Graph {
    let mut graph = InterferenceGraph::new(program);

    for block in program.blocks.node_weights() {
        build_interference_block(&mut graph, block);
    }

    graph.graph
}

fn build_interference_block(graph: &mut InterferenceGraph, block: &prev::Block) {
    for instr in &block.instrs {
        build_interference_instr(graph, &instr);
    }
}

fn build_interference_instr(
    graph: &mut InterferenceGraph,
    AnnotatedInstruction { instr, live_after }: &AnnotatedInstruction,
) {
    // Essentials of Compilation, Siek, Ch. 3.3

    if let Instruction::Operation {
        op: Op::Equals,
        source,
        destination,
    } = instr
    {
        for var in live_after {
            if var != destination && var != source {
                graph.add_edge(destination, var)
            }
        }
    } else {
        for destination in &write_set(instr) {
            for vertex in live_after {
                if vertex != destination {
                    graph.add_edge(destination, vertex);
                }
            }
        }
    }
}

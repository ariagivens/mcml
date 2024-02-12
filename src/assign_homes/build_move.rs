use crate::select_instructions as prev;
use crate::var::Var;
use std::collections::{HashMap, HashSet};

type Index = petgraph::graph::NodeIndex<u32>;
pub type Graph = petgraph::Graph<Var, (), petgraph::Undirected, u32>;

pub struct MoveGraph {
    graph: Graph,
    var_map: HashMap<Var, Index>,
}

impl MoveGraph {
    pub fn move_related(&self, a: &Var, b: &Var) -> bool {
        self.graph.contains_edge(self.var_map[a], self.var_map[b])
    }
}

fn collect_vars(program: &prev::Program) -> HashSet<Var> {
    let mut vs: HashSet<Var> = HashSet::new();

    for b in program.blocks.node_weights() {
        for instr in &b.instrs {
            collect_vars_instr(&mut vs, instr.clone());
        }
    }

    vs
}

fn collect_vars_instr(vs: &mut HashSet<Var>, instr: prev::Instruction) {
    match instr {
        prev::Instruction::Set { var, value } => {
            vs.insert(var);
        }
        prev::Instruction::Operation {
            op,
            source,
            destination,
        } => {
            vs.insert(source);
            vs.insert(destination);
        }
        prev::Instruction::Tellraw { text } => {}
        prev::Instruction::Command { text } => {}
        prev::Instruction::ExecuteIfScoreMatches { var, value, instr } => {
            vs.insert(var);
            collect_vars_instr(vs, *instr);
        }
        prev::Instruction::ExecuteUnlessScoreMatches { var, value, instr } => {
            vs.insert(var);
            collect_vars_instr(vs, *instr);
        }
        prev::Instruction::ExecuteIfScoreEquals { a, b, instr } => {
            vs.insert(a);
            vs.insert(b);
            collect_vars_instr(vs, *instr);
        }
        prev::Instruction::ExecuteUnlessScoreEquals { a, b, instr } => {
            vs.insert(a);
            vs.insert(b);
            collect_vars_instr(vs, *instr);
        }
    }
}

pub fn build_move(program: &prev::Program) -> MoveGraph {
    let mut graph = Graph::new_undirected();
    let mut var_map = HashMap::new();

    for var in collect_vars(program) {
        var_map.insert(var.clone(), graph.add_node(var));
    }

    for block in program.blocks.node_weights() {
        build_move_block(&mut graph, &mut var_map, block);
    }

    MoveGraph { graph, var_map }
}

fn build_move_block(graph: &mut Graph, var_map: &mut HashMap<Var, Index>, block: &prev::Block) {
    for instr in &block.instrs {
        build_move_instr(graph, var_map, &instr);
    }
}

fn build_move_instr(
    graph: &mut Graph,
    var_map: &mut HashMap<Var, Index>,
    instr: &prev::Instruction,
) {
    if let prev::Instruction::Operation {
        op: prev::Op::Equals,
        source,
        destination,
    } = instr
    {
        graph.add_edge(var_map[source], var_map[destination], ());
    }
}

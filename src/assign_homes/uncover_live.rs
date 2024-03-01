use std::collections::{HashMap, HashSet};
use petgraph::Direction;

use crate::linearize::Test;
use crate::select_instructions as prev;
use crate::select_instructions::{Instruction, Jmp};
use crate::var::Var;

use super::{read_set, write_set};

type Graph = petgraph::Graph<Block, (), petgraph::Directed, u32>;

#[derive(Debug)]
pub struct Program {
    pub blocks: Graph,
    pub tests: Vec<Test>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub instrs: Vec<AnnotatedInstruction>,
    live_before: HashSet<Var>,
}

#[derive(Debug, Clone)]
pub struct AnnotatedInstruction {
    pub instr: Instruction,
    pub live_after: HashSet<Var>,
}

pub fn uncover_live(program: &prev::Program) -> Program {
    let mut blocks = petgraph::algo::toposort(&program.blocks, None).unwrap();
    blocks.reverse();
    let mut annotated_blocks: HashMap<prev::Index, Block> = HashMap::new();
    for block in blocks {
        let mut live_after = HashSet::new();
        for jmp in program.blocks.edges_directed(block, Direction::Outgoing) {
            for var in uncover_live_before_jmp(jmp.weight(), &annotated_blocks) {
                live_after.insert(var);
            }
        }
        annotated_blocks.insert(block, uncover_live_block(program.blocks.node_weight(block).unwrap().clone(), live_after));
    }

    let blocks = program
        .blocks
        .map(|idx, _| annotated_blocks[&idx].clone(), |_, e| ());

    Program {
        tests: program.tests.clone(),
        blocks,
    }
}

fn uncover_live_block(block: prev::Block, mut live_after: HashSet<Var>) -> Block {
    let mut instrs = Vec::new();
    for instr in block.instrs.into_iter().rev() {
        instrs.push(AnnotatedInstruction {
            instr: instr.clone(),
            live_after: live_after.clone(),
        });
        live_after = uncover_live_before(instr, &live_after);
    }
    Block { instrs, live_before: live_after }
}

fn uncover_live_before(instr: prev::Instruction, live_after: &HashSet<Var>) -> HashSet<Var> {
    // L_before(k) = (L_after(k) - W(k)) ∪ R(k)
    // Essentials of Compilation, Siek, Eq. 3.3
    let unwritten: HashSet<Var> = live_after.difference(&write_set(&instr)).cloned().collect();
    unwritten.union(&read_set(&instr)).cloned().collect()
}

fn uncover_live_before_jmp(jmp: &Jmp, annotated_blocks: &HashMap<prev::Index, Block>) -> HashSet<Var> {
    match jmp {
        Jmp::ExecuteIfScoreMatchesFunction { var, value: _, block } => {
            let mut live_before = annotated_blocks[&block].live_before.clone();
            live_before.insert(var.clone());
            live_before
        },
        Jmp::ExecuteUnlessScoreMatchesFunction { var, value: _, block } => {
            let mut live_before = annotated_blocks[&block].live_before.clone();
            live_before.insert(var.clone());
            live_before
        },
        Jmp::ExecuteIfScoreEqualsFunction { a, b, block } => {
            let mut live_before = annotated_blocks[&block].live_before.clone();
            live_before.insert(a.clone());
            live_before.insert(b.clone());
            live_before
        },
        Jmp::ExecuteUnlessScoreEqualsFunction { a, b, block } => {
            let mut live_before = annotated_blocks[&block].live_before.clone();
            live_before.insert(a.clone());
            live_before.insert(b.clone());
            live_before
        },
        Jmp::Function { block } => annotated_blocks[&block].live_before.clone(),
    }
}
use std::collections::HashSet;

use crate::linearize::Test;
use crate::select_instructions as prev;
use crate::select_instructions::Instruction;
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
}

#[derive(Debug, Clone)]
pub struct AnnotatedInstruction {
    pub instr: Instruction,
    pub live_after: HashSet<Var>,
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Equals,
    PlusEquals,
    MinusEquals,
    TimesEquals,
    DivideEquals,
}

pub fn uncover_live(program: &prev::Program) -> Program {
    let blocks = program
        .blocks
        .map(|_, b| uncover_live_block(b.clone()), |_, e| *e);

    Program {
        tests: program.tests.clone(),
        blocks,
    }
}

fn uncover_live_block(block: prev::Block) -> Block {
    let mut instrs = Vec::new();
    let mut live_after = HashSet::new();
    for instr in block.instrs.into_iter().rev() {
        instrs.push(AnnotatedInstruction {
            instr: instr.clone(),
            live_after: live_after.clone(),
        });
        live_after = uncover_live_before(instr, &live_after);
    }
    Block { instrs }
}

fn uncover_live_before(instr: prev::Instruction, live_after: &HashSet<Var>) -> HashSet<Var> {
    // L_before(k) = (L_after(k) - W(k)) ∪ R(k)
    // Essentials of Compilation, Siek, Eq. 3.3
    let unwritten: HashSet<Var> = live_after.difference(&write_set(&instr)).cloned().collect();
    unwritten.union(&read_set(&instr)).cloned().collect()
}

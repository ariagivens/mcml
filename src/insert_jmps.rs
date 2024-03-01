use petgraph::{Direction, EdgeDirection};
use crate::assign_homes as prev;
pub use crate::assign_homes::Location;
use crate::select_instructions::Op;

use petgraph::graph::NodeIndex;
use crate::linearize::Test;

type Graph = petgraph::Graph<Block, (), petgraph::Directed, u32>;
type Index = NodeIndex<u32>;

#[derive(Debug)]
pub struct Program {
    pub blocks: Graph,
    pub tests: Vec<Test>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub instrs: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Set {
        location: Location,
        value: i64,
    },
    Operation {
        op: Op,
        source: Location,
        destination: Location,
    },
    Tellraw {
        text: String,
    },
    Command {
        text: String,
    },
    ExecuteIfScoreMatches {
        location: Location,
        value: i64,
        run: Run,
    },
    ExecuteUnlessScoreMatches {
        location: Location,
        value: i64,
        run: Run,
    },
    ExecuteIfScoreEquals {
        a: Location,
        b: Location,
        run: Run,
    },
    ExecuteUnlessScoreEquals {
        a: Location,
        b: Location,
        run: Run,
    },
    Function {
        block: Index,
    }
}

#[derive(Debug, Clone)]
pub enum Run {
    Function { block: Index },
    Set { location: Location, value: i64 },
}

pub fn insert_jmps(program: prev::Program) -> Program {
    let blocks = program.blocks.map(|idx, b| {
        insert_jmps_block(idx, b.clone(), &program.blocks)
    }, |_, _| ());

    Program { blocks, tests: program.tests }
}

fn insert_jmps_block(idx: Index, block: prev::Block, graph: &prev::Graph) -> Block {
    let mut instrs: Vec<Instruction> = block.instrs.into_iter().map(insert_jmps_instr).collect();
    for e in graph.edges_directed(idx, Direction::Outgoing) {
        instrs.push(insert_jmps_jmp(e.weight().clone()));
    }
    Block { instrs }
}

fn insert_jmps_instr(instr: prev::Instruction) -> Instruction {
    match instr {
        prev::Instruction::Set { location, value } => Instruction::Set { location, value },
        prev::Instruction::Operation { op, source, destination } => Instruction::Operation { op, source, destination },
        prev::Instruction::Tellraw { text } => Instruction::Tellraw { text },
        prev::Instruction::Command { text } => Instruction::Command { text },
        prev::Instruction::ExecuteIfScoreMatchesSet { location, value, set_location, set_value } =>
            Instruction::ExecuteIfScoreMatches {
                location,
                value,
                run: Run::Set { location: set_location, value: set_value },
            },
        prev::Instruction::ExecuteUnlessScoreMatchesSet { location, value, set_location, set_value } =>
            Instruction::ExecuteUnlessScoreMatches {
                location,
                value,
                run: Run::Set { location: set_location, value: set_value },
            },
        prev::Instruction::ExecuteIfScoreEqualsSet { a, b, set_location, set_value } =>
            Instruction::ExecuteIfScoreEquals {
                a,
                b,
                run: Run::Set { location: set_location, value: set_value },
            },
        prev::Instruction::ExecuteUnlessScoreEqualsSet { a, b, set_location, set_value } =>
            Instruction::ExecuteUnlessScoreEquals {
                a,
                b,
                run: Run::Set { location: set_location, value: set_value },
            }
    }
}

fn insert_jmps_jmp(jmp: prev::Jmp) -> Instruction {
    match jmp {
        prev::Jmp::ExecuteIfScoreMatchesFunction { location, value, block } => Instruction::ExecuteIfScoreMatches { location, value, run: Run::Function { block } },
        prev::Jmp::ExecuteUnlessScoreMatchesFunction { location, value, block } => Instruction::ExecuteUnlessScoreMatches { location, value, run: Run::Function { block } },
        prev::Jmp::ExecuteIfScoreEqualsFunction { a, b, block } => Instruction::ExecuteIfScoreEquals { a, b, run: Run::Function { block } },
        prev::Jmp::ExecuteUnlessScoreEqualsFunction { a, b, block } => Instruction::ExecuteUnlessScoreEquals { a, b, run: Run::Function { block } },
        prev::Jmp::Function { block } => Instruction::Function { block },
    }
}
mod build_interference;
mod build_move;
mod color_graph;
mod uncover_live;

use crate::linearize::Test;
use crate::select_instructions::{self as prev, Op};
use crate::var::Var;
use build_interference::build_interference;
use build_move::build_move;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use uncover_live::uncover_live;

use self::color_graph::color_graph;

type Graph = petgraph::Graph<Block, (), petgraph::Directed, u32>;

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
        instr: Box<Instruction>,
    },
    ExecuteUnlessScoreMatches {
        location: Location,
        value: i64,
        instr: Box<Instruction>,
    },
    ExecuteIfScoreEquals {
        a: Location,
        b: Location,
        instr: Box<Instruction>,
    },
    ExecuteUnlessScoreEquals {
        a: Location,
        b: Location,
        instr: Box<Instruction>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    Register(Register),
    Stack { offset: u32 },
}

impl Location {
    pub fn from_color(color: u32) -> Self {
        match color {
            0 => Location::Register(Register::R1),
            1 => Location::Register(Register::R2),
            2 => Location::Register(Register::R3),
            3 => Location::Register(Register::R4),
            4 => Location::Register(Register::R5),
            5 => Location::Register(Register::R6),
            6 => Location::Register(Register::R7),
            7 => Location::Register(Register::R8),
            8 => Location::Register(Register::E1),
            9 => Location::Register(Register::E2),
            10 => Location::Register(Register::E3),
            11 => Location::Register(Register::E4),
            12 => Location::Register(Register::E5),
            13 => Location::Register(Register::E6),
            14 => Location::Register(Register::E7),
            15 => Location::Register(Register::E8),
            n => Location::Stack { offset: n - 15 },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Register {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    E1,
    E2,
    E3,
    E4,
    E5,
    E6,
    E7,
    E8,
}

impl Register {
    fn to_color(&self) -> u32 {
        match self {
            Register::R1 => 0,
            Register::R2 => 1,
            Register::R3 => 2,
            Register::R4 => 3,
            Register::R5 => 4,
            Register::R6 => 5,
            Register::R7 => 6,
            Register::R8 => 7,
            Register::E1 => 8,
            Register::E2 => 9,
            Register::E3 => 10,
            Register::E4 => 11,
            Register::E5 => 12,
            Register::E6 => 13,
            Register::E7 => 14,
            Register::E8 => 15,
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Register::R1 => "r1",
                Register::R2 => "r2",
                Register::R3 => "r3",
                Register::R4 => "r4",
                Register::R5 => "r5",
                Register::R6 => "r6",
                Register::R7 => "r7",
                Register::R8 => "r8",
                Register::E1 => "e1",
                Register::E2 => "e2",
                Register::E3 => "e3",
                Register::E4 => "e4",
                Register::E5 => "e5",
                Register::E6 => "e6",
                Register::E7 => "e7",
                Register::E8 => "e8",
            }
        )
    }
}

pub fn assign_homes(program: prev::Program) -> Program {
    let move_graph = build_move(&program);
    let annotated_program = uncover_live(&program);
    let interference_graph = build_interference(&annotated_program);
    let color_map = color_graph(interference_graph, &move_graph);
    let location_map: HashMap<Var, Location> = color_map
        .iter()
        .map(|(var, color)| (var.clone(), Location::from_color(*color)))
        .collect();

    assign_homes_program(program, &location_map)
}

fn assign_homes_program(program: prev::Program, location_map: &HashMap<Var, Location>) -> Program {
    let blocks = program.blocks.map(
        |_, n| assign_homes_block(n.clone(), location_map),
        |_, e| *e,
    );
    Program {
        blocks,
        tests: program.tests,
    }
}

fn assign_homes_block(block: prev::Block, location_map: &HashMap<Var, Location>) -> Block {
    Block {
        instrs: block
            .instrs
            .iter()
            .map(|instr| assign_homes_instr(instr.clone(), location_map))
            .collect(),
    }
}

fn assign_homes_instr(
    instr: prev::Instruction,
    location_map: &HashMap<Var, Location>,
) -> Instruction {
    match instr {
        prev::Instruction::Set { var, value } => Instruction::Set {
            location: location_map[&var].clone(),
            value,
        },
        prev::Instruction::Operation {
            op,
            source,
            destination,
        } => Instruction::Operation {
            op,
            source: location_map[&source].clone(),
            destination: location_map[&destination].clone(),
        },
        prev::Instruction::Tellraw { text } => Instruction::Tellraw { text },
        prev::Instruction::Command { text } => Instruction::Command { text },
        prev::Instruction::ExecuteIfScoreMatches { var, value, instr } => {
            Instruction::ExecuteIfScoreMatches {
                location: location_map[&var].clone(),
                value,
                instr: Box::new(assign_homes_instr(*instr, location_map)),
            }
        }
        prev::Instruction::ExecuteUnlessScoreMatches { var, value, instr } => {
            Instruction::ExecuteUnlessScoreMatches {
                location: location_map[&var].clone(),
                value,
                instr: Box::new(assign_homes_instr(*instr, location_map)),
            }
        }
        prev::Instruction::ExecuteIfScoreEquals { a, b, instr } => {
            Instruction::ExecuteIfScoreEquals {
                a: location_map[&a].clone(),
                b: location_map[&b].clone(),
                instr: Box::new(assign_homes_instr(*instr, location_map)),
            }
        }
        prev::Instruction::ExecuteUnlessScoreEquals { a, b, instr } => {
            Instruction::ExecuteUnlessScoreEquals {
                a: location_map[&a].clone(),
                b: location_map[&b].clone(),
                instr: Box::new(assign_homes_instr(*instr, location_map)),
            }
        }
    }
}

fn write_set(instr: &prev::Instruction) -> HashSet<Var> {
    match instr {
        prev::Instruction::Set { var, value } => HashSet::from([var.clone()]),
        prev::Instruction::Operation {
            op,
            source,
            destination,
        } => HashSet::from([destination.clone()]),
        prev::Instruction::Tellraw { text } => HashSet::new(),
        prev::Instruction::Command { text } => HashSet::new(),
        prev::Instruction::ExecuteIfScoreMatches { var, value, instr } => write_set(instr),
        prev::Instruction::ExecuteUnlessScoreMatches { var, value, instr } => write_set(instr),
        prev::Instruction::ExecuteIfScoreEquals { a, b, instr } => write_set(instr),
        prev::Instruction::ExecuteUnlessScoreEquals { a, b, instr } => write_set(instr),
    }
}

fn read_set(instr: &prev::Instruction) -> HashSet<Var> {
    match instr {
        prev::Instruction::Set { var, value } => HashSet::new(),
        prev::Instruction::Operation {
            op: Op::Equals,
            source,
            destination,
        } => HashSet::from([source.clone()]),
        prev::Instruction::Operation {
            op,
            source,
            destination,
        } => HashSet::from([source.clone(), destination.clone()]),
        prev::Instruction::Tellraw { text } => HashSet::new(),
        prev::Instruction::Command { text } => HashSet::new(),
        prev::Instruction::ExecuteIfScoreMatches { var, value, instr } => {
            HashSet::from([var.clone()])
                .union(&read_set(instr))
                .cloned()
                .collect()
        }
        prev::Instruction::ExecuteUnlessScoreMatches { var, value, instr } => {
            HashSet::from([var.clone()])
                .union(&read_set(instr))
                .cloned()
                .collect()
        }
        prev::Instruction::ExecuteIfScoreEquals { a, b, instr } => {
            HashSet::from([a.clone(), b.clone()])
                .union(&read_set(instr))
                .cloned()
                .collect()
        }
        prev::Instruction::ExecuteUnlessScoreEquals { a, b, instr } => {
            HashSet::from([a.clone(), b.clone()])
                .union(&read_set(instr))
                .cloned()
                .collect()
        }
    }
}

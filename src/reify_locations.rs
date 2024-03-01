use std::fmt::Display;

use crate::linearize::Test;
use crate::select_instructions::Op;
type Graph = petgraph::Graph<Block, (), petgraph::Directed, u32>;
type Index = petgraph::graph::NodeIndex<u32>;

use crate::assign_homes::Register;
use crate::insert_jmps as prev;

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
    Push {
        offset: u32,
    },
    Pop {
        offset: u32,
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

#[derive(Clone, Debug)]
pub enum Location {
    Register(Register),
    StackItem,
    Scratch,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Register(r) => write!(f, "{r} registry"),
            Location::StackItem => write!(f, "item stack"),
            Location::Scratch => write!(f, "scratch registry"),
        }
    }
}

pub fn reify_location(program: prev::Program) -> Program {
    let blocks = program
        .blocks
        .map(|_, n| reify_location_block(n.clone()), |_, e| *e);
    Program {
        blocks,
        tests: program.tests,
    }
}

fn reify_location_block(block: prev::Block) -> Block {
    let instrs = block
        .instrs
        .into_iter()
        .map(|instr| reify_location_instr(instr))
        .flatten()
        .collect();
    Block { instrs }
}

fn reify_location_instr(instr: prev::Instruction) -> Vec<Instruction> {
    match instr {
        prev::Instruction::Set {
            location: prev::Location::Register(r),
            value,
        } => vec![Instruction::Set {
            location: Location::Register(r),
            value,
        }],
        prev::Instruction::Set {
            location: prev::Location::Stack { offset },
            value,
        } => vec![
            Instruction::Set {
                location: Location::StackItem,
                value,
            },
            Instruction::Push { offset },
        ],
        prev::Instruction::Operation {
            op: Op::Equals,
            source,
            destination,
        } if source == destination => vec![],
        prev::Instruction::Operation {
            op,
            source: prev::Location::Register(source),
            destination: prev::Location::Register(destination),
        } => vec![Instruction::Operation {
            op,
            source: Location::Register(source),
            destination: Location::Register(destination),
        }],
        prev::Instruction::Operation {
            op,
            source: prev::Location::Stack { offset },
            destination: prev::Location::Register(destination),
        } => vec![
            Instruction::Pop { offset },
            Instruction::Operation {
                op,
                source: Location::StackItem,
                destination: Location::Register(destination),
            },
        ],
        prev::Instruction::Operation {
            op,
            source: prev::Location::Register(source),
            destination: prev::Location::Stack { offset },
        } => vec![
            Instruction::Pop { offset },
            Instruction::Operation {
                op,
                source: Location::Register(source),
                destination: Location::StackItem,
            },
            Instruction::Push { offset },
        ],
        prev::Instruction::Operation {
            op,
            source: prev::Location::Stack {
                offset: source_offset,
            },
            destination:
                prev::Location::Stack {
                    offset: destination_offset,
                },
        } => vec![
            Instruction::Pop {
                offset: source_offset,
            },
            Instruction::Operation {
                op: Op::Equals,
                source: Location::StackItem,
                destination: Location::Scratch,
            },
            Instruction::Pop {
                offset: destination_offset,
            },
            Instruction::Operation {
                op,
                source: Location::Scratch,
                destination: Location::StackItem,
            },
            Instruction::Push {
                offset: destination_offset,
            },
        ],
        prev::Instruction::Tellraw { text } => vec![Instruction::Tellraw { text }],
        prev::Instruction::Command { text } => vec![Instruction::Command { text }],
        prev::Instruction::ExecuteIfScoreMatches {
            location: prev::Location::Register(r),
            value,
            run,
        } => {
            let (run, after) = reify_location_run(run);
            let mut instrs = vec![Instruction::ExecuteIfScoreMatches {
                location: Location::Register(r),
                value,
                run,
            }];
            instrs.extend(after);
            instrs
        }
        prev::Instruction::ExecuteIfScoreMatches {
            location: prev::Location::Stack { offset },
            value,
            run,
        } => {
            let (run, after) = reify_location_run(run);
            let mut instrs = vec![
                Instruction::Pop { offset },
                Instruction::ExecuteIfScoreMatches {
                    location: Location::StackItem,
                    value,
                    run
                },
            ];
            instrs.extend(after);
            instrs
        }
        prev::Instruction::ExecuteUnlessScoreMatches {
            location: prev::Location::Register(r),
            value,
            run,
        } => {
            let (run, after) = reify_location_run(run);
            let mut instrs = vec![Instruction::ExecuteUnlessScoreMatches {
                    location: Location::Register(r),
                    value,
                    run,
                }];
            instrs.extend(after);
            instrs
        }
        prev::Instruction::ExecuteUnlessScoreMatches {
            location: prev::Location::Stack { offset },
            value,
            run
        } => {
            let (run, after) = reify_location_run(run);
            let mut instrs = vec![
                    Instruction::Pop { offset },
                    Instruction::ExecuteUnlessScoreMatches {
                        location: Location::StackItem,
                        value,
                        run,
                    },
                ];
            instrs.extend(after);
            instrs
        }
        prev::Instruction::ExecuteIfScoreEquals { a, b, run } => todo!(),
        prev::Instruction::ExecuteUnlessScoreEquals { a, b, run } => todo!(),
        prev::Instruction::Function { block } => vec![ Instruction::Function { block } ],
    }
}

fn reify_location_run(run: prev::Run) -> (Run, Vec<Instruction>) {
    match run {
        prev::Run::Function { block } => (Run::Function { block }, Vec::new()),
        prev::Run::Set { location: prev::Location::Register(r), value } => (Run::Set { location: Location::Register(r), value }, Vec::new()),
        prev::Run::Set { location: prev::Location::Stack { offset }, value } => (Run::Set { location: Location::StackItem, value }, vec![Instruction::Push { offset }])
    }
}
use std::fmt::Display;

use crate::linearize::{self as prev, Test};
use crate::var::{Var, VarFactory};

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
        var: Var,
        value: i64,
    },
    Operation {
        op: Op,
        source: Var,
        destination: Var,
    },
    Tellraw {
        text: String,
    },
    Command {
        text: String,
    },
    ExecuteIfScoreMatches {
        var: Var,
        value: i64,
        instr: Box<Instruction>,
    },
    ExecuteUnlessScoreMatches {
        var: Var,
        value: i64,
        instr: Box<Instruction>,
    },
    ExecuteIfScoreEquals {
        a: Var,
        b: Var,
        instr: Box<Instruction>,
    },
    ExecuteUnlessScoreEquals {
        a: Var,
        b: Var,
        instr: Box<Instruction>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Equals,
    PlusEquals,
    MinusEquals,
    TimesEquals,
    DivideEquals,
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Op::Equals => "=",
            Op::PlusEquals => "+=",
            Op::MinusEquals => "-=",
            Op::TimesEquals => "*=",
            Op::DivideEquals => "/=",
        })
    }
}

pub fn select_instructions(mut program: prev::Program) -> Program {
    let blocks = program.blocks.map(
        |_, b| select_instructions_block(b.clone(), &mut program.var_factory),
        |_, e| *e,
    );

    Program {
        blocks,
        tests: program.tests,
    }
}

fn select_instructions_block(block: prev::Block, var_factory: &mut VarFactory) -> Block {
    let mut instrs = Vec::new();

    for stmt in block.stmts {
        instrs.extend(select_instructions_stmt(stmt, var_factory));
    }

    Block { instrs }
}

fn select_instructions_stmt(
    stmt: prev::Statement,
    var_factory: &mut VarFactory,
) -> Vec<Instruction> {
    match stmt {
        prev::Statement::Assign {
            var,
            expr: prev::Expr::Atom(prev::Atom::LitInt(value)),
        } => vec![Instruction::Set { var, value }],
        prev::Statement::Assign {
            var,
            expr: prev::Expr::Atom(prev::Atom::LitBool(value)),
        } => vec![Instruction::Set {
            var,
            value: if value { 1 } else { 0 },
        }],
        prev::Statement::Assign {
            var: destination,
            expr: prev::Expr::Atom(prev::Atom::Var(source)),
        } => vec![Instruction::Operation {
            op: Op::Equals,
            source,
            destination,
        }],
        prev::Statement::Assign {
            var,
            expr: prev::Expr::Binary { left, right, op },
        } => {
            let mut instrs = Vec::new();
            instrs.push(match left {
                prev::Atom::LitInt(value) => Instruction::Set {
                    var: var.clone(),
                    value,
                },
                prev::Atom::LitBool(value) => Instruction::Set {
                    var: var.clone(),
                    value: if value { 1 } else { 0 },
                },
                prev::Atom::Var(left) => Instruction::Operation {
                    op: Op::Equals,
                    source: left,
                    destination: var.clone(),
                },
            });
            instrs.extend(match right {
                prev::Atom::LitInt(value) => {
                    let tmp = var_factory.tmp();
                    vec![
                        Instruction::Set {
                            var: tmp.clone(),
                            value,
                        },
                        Instruction::Operation {
                            op: op_assign(op),
                            source: tmp,
                            destination: var,
                        },
                    ]
                }
                prev::Atom::LitBool(value) => {
                    let tmp = var_factory.tmp();
                    vec![
                        Instruction::Set {
                            var: tmp.clone(),
                            value: if value { 1 } else { 0 },
                        },
                        Instruction::Operation {
                            op: op_assign(op),
                            source: tmp,
                            destination: var,
                        },
                    ]
                }
                prev::Atom::Var(right) => {
                    vec![Instruction::Operation {
                        op: op_assign(op),
                        source: right,
                        destination: var,
                    }]
                }
            });
            instrs
        }
        prev::Statement::Assert {
            atom: prev::Atom::LitInt(_),
        } => panic!("Assert int?"),
        prev::Statement::Assert {
            atom: prev::Atom::LitBool(b),
        } => vec![Instruction::Tellraw {
            text: if b {
                "ok".to_owned()
            } else {
                "not ok".to_owned()
            },
        }],
        prev::Statement::Assert {
            atom: prev::Atom::Var(var),
        } => vec![
            Instruction::ExecuteIfScoreMatches {
                var: var.clone(),
                value: 1,
                instr: Box::new(Instruction::Tellraw {
                    text: "ok".to_owned(),
                }),
            },
            Instruction::ExecuteUnlessScoreMatches {
                var,
                value: 1,
                instr: Box::new(Instruction::Tellraw {
                    text: "not ok".to_owned(),
                }),
            },
        ],
        prev::Statement::AssertEq {
            left: prev::Atom::LitBool(a),
            right: prev::Atom::LitBool(b),
        } => vec![Instruction::Tellraw {
            text: if a == b {
                "ok".to_owned()
            } else {
                "not ok".to_owned()
            },
        }],
        prev::Statement::AssertEq {
            left: prev::Atom::LitInt(a),
            right: prev::Atom::LitInt(b),
        } => vec![Instruction::Tellraw {
            text: if a == b {
                "ok".to_owned()
            } else {
                "not ok".to_owned()
            },
        }],
        prev::Statement::AssertEq {
            left: prev::Atom::Var(var),
            right: prev::Atom::LitInt(value),
        }
        | prev::Statement::AssertEq {
            left: prev::Atom::LitInt(value),
            right: prev::Atom::Var(var),
        } => vec![
            Instruction::ExecuteIfScoreMatches {
                var: var.clone(),
                value,
                instr: Box::new(Instruction::Tellraw {
                    text: "ok".to_owned(),
                }),
            },
            Instruction::ExecuteUnlessScoreMatches {
                var,
                value,
                instr: Box::new(Instruction::Tellraw {
                    text: "not ok".to_owned(),
                }),
            },
        ],
        prev::Statement::AssertEq {
            left: prev::Atom::Var(var),
            right: prev::Atom::LitBool(value),
        }
        | prev::Statement::AssertEq {
            left: prev::Atom::LitBool(value),
            right: prev::Atom::Var(var),
        } => vec![
            Instruction::ExecuteIfScoreMatches {
                var: var.clone(),
                value: if value { 1 } else { 0 },
                instr: Box::new(Instruction::Tellraw {
                    text: "ok".to_owned(),
                }),
            },
            Instruction::ExecuteUnlessScoreMatches {
                var,
                value: if value { 1 } else { 0 },
                instr: Box::new(Instruction::Tellraw {
                    text: "not ok".to_owned(),
                }),
            },
        ],
        prev::Statement::AssertEq {
            left: prev::Atom::Var(a),
            right: prev::Atom::Var(b),
        } => vec![
            Instruction::ExecuteIfScoreEquals {
                a: a.clone(),
                b: b.clone(),
                instr: Box::new(Instruction::Tellraw {
                    text: "ok".to_owned(),
                }),
            },
            Instruction::ExecuteUnlessScoreEquals {
                a,
                b,
                instr: Box::new(Instruction::Tellraw {
                    text: "ok".to_owned(),
                }),
            },
        ],
        prev::Statement::AssertEq { .. } => panic!("Type error!"),
        prev::Statement::Command { text } => vec![Instruction::Command { text }],
    }
}

fn op_assign(op: prev::Op) -> Op {
    match op {
        prev::Op::Plus => Op::PlusEquals,
        prev::Op::Minus => Op::MinusEquals,
        prev::Op::Times => Op::TimesEquals,
        prev::Op::Divide => Op::DivideEquals,
    }
}

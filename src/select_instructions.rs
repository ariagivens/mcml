use std::collections::HashMap;
use std::fmt::Display;
use petgraph::data::DataMap;
use petgraph::Direction;

use crate::linearize::{self as prev, Atom, Cmp, Statement, Test};
use crate::select_instructions::Instruction::Tellraw;
use crate::var::{Var, VarFactory};

pub type Graph = petgraph::Graph<Block, Jmp, petgraph::Directed, u32>;
pub type Index = petgraph::graph::NodeIndex<u32>;

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
pub enum Jmp {
    ExecuteIfScoreMatchesFunction {
        var: Var,
        value: i64,
        block: Index,
    },
    ExecuteUnlessScoreMatchesFunction {
        var: Var,
        value: i64,
        block: Index,
    },
    ExecuteIfScoreEqualsFunction {
        a: Var,
        b: Var,
        block: Index,
    },
    ExecuteUnlessScoreEqualsFunction {
        a: Var,
        b: Var,
        block: Index,
    },
    Function {
        block: Index,
    }
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
    ExecuteIfScoreMatchesSet {
        var: Var,
        value: i64,
        set_var: Var,
        set_value: i64,
    },
    ExecuteUnlessScoreMatchesSet {
        var: Var,
        value: i64,
        set_var: Var,
        set_value: i64,
    },
    ExecuteIfScoreEqualsSet {
        a: Var,
        b: Var,
        set_var: Var,
        set_value: i64,
    },
    ExecuteUnlessScoreEqualsSet {
        a: Var,
        b: Var,
        set_var: Var,
        set_value: i64,
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
        write!(
            f,
            "{}",
            match self {
                Op::Equals => "=",
                Op::PlusEquals => "+=",
                Op::MinusEquals => "-=",
                Op::TimesEquals => "*=",
                Op::DivideEquals => "/=",
            }
        )
    }
}

pub fn select_instructions(mut program: prev::Program) -> Program {
    let blocks = program.blocks.filter_map(
        |_, b|
            Some(select_instructions_block(b.clone(), &mut program.var_factory))
        ,
        |idx, e| {
            let (_, target) = program.blocks.edge_endpoints(idx).unwrap();
            select_instructions_jmp(e, target)
        },
    );

    Program {
        blocks,
        tests: program.tests,
    }
}

fn select_instructions_jmp(jmp: &prev::Jmp, block: Index) -> Option<Jmp> {
    match jmp {
        prev::Jmp::Unconditional => Some(Jmp::Function { block }),
        prev::Jmp::If(prev::Condition::Atm(Atom::LitBool(b))) => if *b { Some(Jmp::Function { block }) } else { None },
        prev::Jmp::If(prev::Condition::Atm(Atom::Var(var))) => Some(Jmp::ExecuteIfScoreMatchesFunction {
            var: var.clone(),
            value: 1,
            block,
        }),
        prev::Jmp::If(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::LitBool(a), right: Atom::LitBool(b), }) =>
            if *a == *b { Some(Jmp::Function { block }) } else { None },
        prev::Jmp::If(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::LitInt(a), right: Atom::LitInt(b), }) =>
            if *a == *b { Some(Jmp::Function { block }) } else { None },
        prev::Jmp::If(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::Var(var), right: Atom::LitBool(b), } | prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::LitBool(b), right: Atom::Var(var), }) =>
            Some(Jmp::ExecuteIfScoreMatchesFunction {
                var: var.clone(),
                value: if *b { 1 } else { 0 },
                block,
            }),
        prev::Jmp::If(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::Var(var), right: Atom::LitInt(i), } | prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::LitInt(i), right: Atom::Var(var), }) =>
            Some(Jmp::ExecuteIfScoreMatchesFunction {
                var: var.clone(),
                value: *i,
                block,
            }),
        prev::Jmp::If(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::Var(left), right: Atom::Var(right) }) =>
            Some(Jmp::ExecuteIfScoreEqualsFunction {
                a: left.clone(),
                b: right.clone(),
                block,
            }),
        prev::Jmp::If(_) => panic!("Type error!"),
        prev::Jmp::Unless(prev::Condition::Atm(Atom::LitBool(b))) => if !*b { Some(Jmp::Function { block }) } else { None },
        prev::Jmp::Unless(prev::Condition::Atm(Atom::Var(var))) => Some(Jmp::ExecuteUnlessScoreMatchesFunction {
            var: var.clone(),
            value: 1,
            block,
        }),
        prev::Jmp::Unless(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::LitBool(a), right: Atom::LitBool(b), }) =>
            if *a != *b { Some(Jmp::Function { block }) } else { None },
        prev::Jmp::Unless(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::LitInt(a), right: Atom::LitInt(b), }) =>
            if *a != *b { Some(Jmp::Function { block }) } else { None },
        prev::Jmp::Unless(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::Var(var), right: Atom::LitBool(b), } | prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::LitBool(b), right: Atom::Var(var), }) =>
            Some(Jmp::ExecuteUnlessScoreMatchesFunction {
                var: var.clone(),
                value: if *b { 1 } else { 0 },
                block,
            }),
        prev::Jmp::Unless(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::Var(var), right: Atom::LitInt(i), } | prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::LitInt(i), right: Atom::Var(var), }) =>
            Some(Jmp::ExecuteUnlessScoreMatchesFunction {
                var: var.clone(),
                value: *i,
                block,
            }),
        prev::Jmp::Unless(prev::Condition::Cmp { cmp: Cmp::Eq, left: Atom::Var(left), right: Atom::Var(right) }) =>
            Some(Jmp::ExecuteUnlessScoreEqualsFunction {
                a: left.clone(),
                b: right.clone(),
                block,
            }),
        prev::Jmp::Unless(_) => panic!("Type error!"),
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
        prev::Statement::TellOk { test_name } => vec![Tellraw { text: format!("ok - {test_name}") }],
        prev::Statement::TellNotOk { test_name } => vec![Tellraw { text: format!("not ok - {test_name}") }],
        prev::Statement::Assign {
            var,
            expr: prev::Expr::Atom(prev::Atom::LitUnit),
        } => Vec::new(),
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
        prev::Statement::Assign { var, expr: prev::Expr::Cmp { cmp: Cmp::Eq, left, right } } =>
            match (left, right) {
                (Atom::LitInt(l), Atom::LitInt(r)) => vec![
                    Instruction::Set { var, value: if l == r { 1 } else { 0 } }
                ],
                (Atom::LitBool(l), Atom::LitBool(r)) => vec![
                    Instruction::Set { var, value: if l == r { 1 } else { 0 }}
                ],
                (Atom::Var(v), Atom::LitInt(i)) | (Atom::LitInt(i), Atom::Var(v)) => vec![
                    Instruction::ExecuteIfScoreMatchesSet {
                        var: v.clone(),
                        value: i,
                        set_var: var.clone(),
                        set_value: 1,
                    },
                    Instruction::ExecuteUnlessScoreMatchesSet {
                        var: v,
                        value: i,
                        set_var: var,
                        set_value: 0,
                    }
                ],
                (Atom::Var(v), Atom::LitBool(b)) | (Atom::LitBool(b), Atom::Var(v)) => vec![
                    Instruction::ExecuteIfScoreMatchesSet {
                        var: v.clone(),
                        value: if b { 1 } else { 0 },
                        set_var: var.clone(),
                        set_value: 1,
                    },
                    Instruction::ExecuteUnlessScoreMatchesSet {
                        var: v,
                        value: if b { 1 } else { 0 },
                        set_var: var,
                        set_value: 0,
                    }
                ],
                (Atom::Var(left), Atom::Var(right)) => vec![
                    Instruction::ExecuteIfScoreEqualsSet {
                        a: left.clone(),
                        b: right.clone(),
                        set_var: var.clone(),
                        set_value: 1,
                    },
                    Instruction::ExecuteUnlessScoreEqualsSet {
                        a: left,
                        b: right,
                        set_var: var,
                        set_value: 0,
                    }
                ],
                _ => panic!(),
            }
        prev::Statement::Assign {
            var,
            expr: prev::Expr::Binary { left, right, op },
        } => {
            let mut instrs = Vec::new();
            instrs.push(match left {
                prev::Atom::LitUnit => panic!("Type error! Tried to add with unit"),
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
                prev::Atom::LitUnit => panic!("Type error! Tried to add with unit"),
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

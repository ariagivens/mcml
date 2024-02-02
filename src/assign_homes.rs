use std::collections::HashMap;

use crate::{
    linearize::{self, Test},
    var::Var,
};

type Graph = petgraph::Graph<Block, (), petgraph::Directed, u32>;
type Index = petgraph::graph::NodeIndex<u32>;

#[derive(Debug)]
pub struct Program {
    pub blocks: Graph,
    pub tests: Vec<Test>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Statement>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Statement {
    Assign { location: Location, expr: Expr },
    Assert { atom: Atom },
    AssertEq { left: Atom, right: Atom },
    Command { text: String },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Location {
    Stack { offset: usize },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expr {
    Atom(Atom),
    Plus(Binary),
    Minus(Binary),
    Times(Binary),
    Divide(Binary),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Binary {
    pub left: Atom,
    pub right: Atom,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Atom {
    Location(Location),
    LitInt(i64),
    LitBool(bool),
}

pub fn assign_homes(program: linearize::Program) -> Program {
    let linearize::Program { blocks, tests } = program;

    let blocks = blocks.map(|i, n| assign_homes_block(n.clone()), |i, e| *e);

    Program { blocks, tests }
}

struct Stacker {
    map: HashMap<u32, usize>,
    highest: usize,
}

impl Stacker {
    fn new() -> Self {
        Stacker {
            map: HashMap::new(),
            highest: 0,
        }
    }

    fn get(&mut self, id: u32) -> usize {
        match self.map.get(&id) {
            Some(v) => *v,
            None => {
                self.highest += 1;
                self.map.insert(id, self.highest);
                self.highest
            }
        }
    }
}

fn assign_homes_block(block: linearize::Block) -> Block {
    let mut stacker = Stacker::new();

    let mut stmts = Vec::new();
    for stmt in block.stmts {
        stmts.push(match stmt {
            linearize::Statement::Assign { var, expr } => Statement::Assign {
                location: assign_homes_var(&mut stacker, var),
                expr: assign_homes_expr(&mut stacker, expr),
            },
            linearize::Statement::Assert { atom } => Statement::Assert {
                atom: assign_homes_atom(&mut stacker, atom),
            },
            linearize::Statement::AssertEq { left, right } => Statement::AssertEq {
                left: assign_homes_atom(&mut stacker, left),
                right: assign_homes_atom(&mut stacker, right),
            },
            linearize::Statement::Command { text } => Statement::Command { text },
        });
    }

    Block { stmts }
}

fn assign_homes_var(stacker: &mut Stacker, var: Var) -> Location {
    Location::Stack {
        offset: stacker.get(var.id),
    }
}

fn assign_homes_atom(stacker: &mut Stacker, atom: linearize::Atom) -> Atom {
    match atom {
        linearize::Atom::Var(var) => Atom::Location(assign_homes_var(stacker, var)),
        linearize::Atom::LitInt(i) => Atom::LitInt(i),
        linearize::Atom::LitBool(b) => Atom::LitBool(b),
    }
}

fn assign_homes_expr(stacker: &mut Stacker, expr: linearize::Expr) -> Expr {
    match expr {
        linearize::Expr::Atom(atom) => Expr::Atom(assign_homes_atom(stacker, atom)),
        linearize::Expr::Plus(b) => Expr::Plus(assign_homes_binary(stacker, b)),
        linearize::Expr::Minus(b) => Expr::Minus(assign_homes_binary(stacker, b)),
        linearize::Expr::Times(b) => Expr::Times(assign_homes_binary(stacker, b)),
        linearize::Expr::Divide(b) => Expr::Divide(assign_homes_binary(stacker, b)),
    }
}

fn assign_homes_binary(
    stacker: &mut Stacker,
    linearize::Binary { left, right }: linearize::Binary,
) -> Binary {
    Binary {
        left: assign_homes_atom(stacker, left),
        right: assign_homes_atom(stacker, right),
    }
}

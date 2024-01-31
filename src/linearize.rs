use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

use anyhow::Result;

use crate::parse;

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
    // tail: Tail,
}

// struct Tail {
//     expr: Expr
// }

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Statement {
    Assign { var: Var, expr: Expr },
    Assert { atom: Atom },
    AssertEq { left: Atom, right: Atom },
    Command { text: String },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Var {
    pub name: Option<String>,
    pub id: u32,
}

impl Var {
    fn tmp() -> Self {
        static NEXT_VAR_ID: Lazy<Arc<Mutex<u32>>> = Lazy::new(|| Arc::new(Mutex::new(0)));

        let mut next_var_id = NEXT_VAR_ID.lock();
        let id = *next_var_id;
        *next_var_id += 1;
        Var { name: None, id }
    }
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
    Var(Var),
    LitInt(i64),
    LitBool(bool),
}

#[derive(Debug)]
pub struct Test {
    pub name: String,
    pub block: Index,
}

pub fn linearize(defs: Vec<parse::Definition>) -> Program {
    let mut blocks = Graph::new();
    let mut tests = Vec::new();

    for def in defs {
        let parse::Definition::Test { name, stmt } = def;
        tests.push(Test {
            name,
            block: linearize_stmt(&mut blocks, stmt),
        });
    }

    Program { blocks, tests }
}

fn linearize_stmt(blocks: &mut Graph, stmt: parse::Statement) -> Index {
    let stmts = match stmt {
        parse::Statement::Assert { expr } => {
            let (atom, mut stmts) = linearize_expr(expr);
            stmts.push(Statement::Assert { atom });
            stmts
        }
        parse::Statement::AssertEq { left, right } => {
            let (left, mut stmts) = linearize_expr(left);
            let (right, right_stmts) = linearize_expr(right);
            stmts.extend(right_stmts);
            stmts.push(Statement::AssertEq { left, right });
            stmts
        }
        parse::Statement::Command { text } => {
            vec![Statement::Command { text }]
        }
    };

    let block = Block { stmts };

    blocks.add_node(block)
}

fn linearize_expr(expr: parse::Expr) -> (Atom, Vec<Statement>) {
    match expr {
        parse::Expr::LitBool(b) => (Atom::LitBool(b), vec![]),
        parse::Expr::LitInt(i) => (Atom::LitInt(i), vec![]),
        parse::Expr::Plus { left, right } => linearize_binary(Expr::Plus, *left, *right),
        parse::Expr::Minus { left, right } => linearize_binary(Expr::Minus, *left, *right),
        parse::Expr::Times { left, right } => linearize_binary(Expr::Times, *left, *right),
        parse::Expr::Divide { left, right } => linearize_binary(Expr::Divide, *left, *right),
    }
}

fn linearize_binary(
    op: impl Fn(Binary) -> Expr,
    left: parse::Expr,
    right: parse::Expr,
) -> (Atom, Vec<Statement>) {
    let (left, mut stmts) = linearize_expr(left);
    let (right, right_stmts) = linearize_expr(right);
    stmts.extend(right_stmts);

    let var = Var::tmp();
    stmts.push(Statement::Assign {
        var: var.clone(),
        expr: op(Binary { left, right }),
    });

    (Atom::Var(var), stmts)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn assert_literal() {
        let def = parse::Definition::Test {
            name: "test".to_owned(),
            stmt: parse::Statement::Assert {
                expr: parse::Expr::LitBool(true),
            },
        };
        let program = linearize(vec![def]);
        let block = program.blocks[program.tests.first().unwrap().block].clone();

        assert_eq!(
            Block {
                stmts: vec![Statement::Assert {
                    atom: Atom::LitBool(true)
                }]
            },
            block
        );
    }

    #[test]
    fn assert_eq_literal() {
        let def = parse::Definition::Test {
            name: "test".to_owned(),
            stmt: parse::Statement::AssertEq {
                left: parse::Expr::LitInt(11),
                right: parse::Expr::LitInt(12),
            },
        };
        let program = linearize(vec![def]);
        let block = program.blocks[program.tests.first().unwrap().block].clone();

        assert_eq!(
            Block {
                stmts: vec![Statement::AssertEq {
                    left: Atom::LitInt(11),
                    right: Atom::LitInt(12)
                }]
            },
            block
        );
    }

    #[test]
    fn command() {
        let def = parse::Definition::Test {
            name: "test".to_owned(),
            stmt: parse::Statement::Command {
                text: "command text".to_owned(),
            },
        };
        let program = linearize(vec![def]);
        let block = program.blocks[program.tests.first().unwrap().block].clone();

        assert_eq!(
            Block {
                stmts: vec![Statement::Command {
                    text: "command text".to_owned()
                }]
            },
            block
        );
    }

    #[test]
    fn reduce_complex_expression() {
        // (+ (* 1 (- 2 3)) (/ 4 5))
        let def = parse::Definition::Test {
            name: "test".to_owned(),
            stmt: parse::Statement::Assert {
                expr: parse::Expr::Plus {
                    left: Box::new(parse::Expr::Times {
                        left: Box::new(parse::Expr::LitInt(1)),
                        right: Box::new(parse::Expr::Minus {
                            left: Box::new(parse::Expr::LitInt(2)),
                            right: Box::new(parse::Expr::LitInt(3)),
                        }),
                    }),
                    right: Box::new(parse::Expr::Divide {
                        left: Box::new(parse::Expr::LitInt(4)),
                        right: Box::new(parse::Expr::LitInt(5)),
                    }),
                },
            },
        };

        let program = linearize(vec![def]);
        let block = program.blocks[program.tests.first().unwrap().block].clone();
        let stmts = block.stmts;

        // tmp1 = (- 2 3)
        let tmp1 = if let Statement::Assign { var: tmp1, expr: Expr::Minus(Binary { left, right }) } = &stmts[0] {
            assert_eq!(left, &Atom::LitInt(2));
            assert_eq!(right, &Atom::LitInt(3));
            tmp1.clone()
        } else {
            panic!("Expected tmp1 = (- 2 3)");
        };

        // tmp2 = (* 1 tmp1)
        let tmp2 = if let Statement::Assign { var: tmp2, expr: Expr::Times(Binary { left, right }) } = &stmts[1] {
            assert_eq!(left, &Atom::LitInt(1));
            assert_eq!(right, &Atom::Var(tmp1));
            tmp2.clone()
        } else {
            panic!("Expected tmp2 = (* 1 tmp1)");
        };

        // tmp3 = (/ 4 5)
        let tmp3 = if let Statement::Assign { var: tmp3, expr: Expr::Divide(Binary { left, right }) } = &stmts[2] {
            assert_eq!(left, &Atom::LitInt(4));
            assert_eq!(right, &Atom::LitInt(5));
            tmp3.clone()
        } else {
            panic!("Expected tmp3 = (/ 4 5)");
        };

        // tmp4 = (+ tmp2 tmp3)
        let tmp4 = if let Statement::Assign { var: tmp4, expr: Expr::Plus(Binary { left, right }) } = &stmts[3] {
            assert_eq!(left, &Atom::Var(tmp2));
            assert_eq!(right, &Atom::Var(tmp3));
            tmp4.clone()
        } else {
            panic!("Expected tmp2 = (+ tmp2 tmp3)");
        };

        // assert tmp4
        assert_eq!(Statement::Assert { atom: Atom::Var(tmp4) }, stmts[4]);
    }
}

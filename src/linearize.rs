use std::collections::VecDeque;

use itertools::Itertools;

use crate::desugar_asserts as prev;
use crate::var::{Var, VarFactory};

pub type Graph = petgraph::Graph<Block, Jmp, petgraph::Directed, u32>;
pub type Index = petgraph::graph::NodeIndex<u32>;

pub struct Program {
    pub blocks: Graph,
    pub tests: Vec<Test>,
    pub var_factory: VarFactory,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Jmp {
    Unconditional,
    If(Condition),
    Unless(Condition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Cmp { cmp: Cmp, left: Atom, right: Atom },
    Atm(Atom)
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Statement {
    Assign { var: Var, expr: Expr },
    TellOk { test_name: String },
    TellNotOk { test_name: String },
    Command { text: String },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expr {
    Atom(Atom),
    Binary { op: Op, left: Atom, right: Atom },
    Cmp { cmp: Cmp, left: Atom, right: Atom },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Atom {
    Var(Var),
    LitUnit,
    LitInt(i64),
    LitBool(bool),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Op {
    Plus,
    Minus,
    Times,
    Divide,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Cmp {
    Eq,
}

#[derive(Debug, Clone)]
pub struct Test {
    pub name: String,
    pub block: Index,
}

pub fn linearize(
    prev::Program {
        defs,
        mut var_factory,
    }: prev::Program,
) -> Program {
    let mut blocks = Graph::new();
    let mut tests = Vec::new();

    for def in defs {
        let prev::Definition::Test { name, stmts } = def;

        tests.push(Test {
            name,
            block: linearize_stmts(&mut var_factory, &mut blocks, stmts),
        });
    }

    Program {
        blocks,
        tests,
        var_factory,
    }
}

fn linearize_stmts(
    var_factory: &mut VarFactory,
    blocks: &mut Graph,
    stmts: Vec<prev::Statement>,
) -> Index {
    let begin = blocks.add_node(Block { stmts: Vec::new() });
    let mut current = begin;

    for stmt in stmts {
        linearize_stmt(var_factory, blocks, &mut current, stmt);
    }

    begin
}

fn linearize_stmt(var_factory: &mut VarFactory, blocks: &mut Graph, current: &mut Index, stmt: prev::Statement) {
    match stmt {
        prev::Statement::Expr(expr) => {
            let _ = linearize_expr(var_factory, blocks, current, expr);
        },
        prev::Statement::TellOk { test_name } => {
            let stmts = &mut blocks.node_weight_mut(*current).unwrap().stmts;
            stmts.push(Statement::TellOk { test_name });
        },
        prev::Statement::TellNotOk { test_name } => {
            let stmts = &mut blocks.node_weight_mut(*current).unwrap().stmts;
            stmts.push(Statement::TellNotOk { test_name });
        },
        prev::Statement::Command { text } => {
            let stmts = &mut blocks.node_weight_mut(*current).unwrap().stmts;
            stmts.push(Statement::Command { text });
        }
        prev::Statement::Let { var, expr } => {
            let atom = linearize_expr(var_factory, blocks, current, expr);
            let stmts = &mut blocks.node_weight_mut(*current).unwrap().stmts;
            stmts.push(Statement::Assign {
                var,
                expr: Expr::Atom(atom),
            });
        }
    }
}

fn linearize_expr(var_factory: &mut VarFactory, blocks: &mut Graph, current: &mut Index, expr: prev::Expr) -> Atom {
    match expr {
        prev::Expr::LitUnit => Atom::LitUnit,
        prev::Expr::Bundle { stmts, expr } => {
            for stmt in stmts {
                linearize_stmt(var_factory, blocks, current, stmt)
            }
            linearize_expr(var_factory, blocks, current, *expr)
        },
        prev::Expr::LitBool(b) => Atom::LitBool(b),
        prev::Expr::LitInt(i) => Atom::LitInt(i),
        prev::Expr::Variable(x) => Atom::Var(x),
        prev::Expr::Plus { left, right } => linearize_binary(var_factory, blocks, current, Op::Plus, *left, *right),
        prev::Expr::Minus { left, right } => {
            linearize_binary(var_factory, blocks, current, Op::Minus, *left, *right)
        }
        prev::Expr::Times { left, right } => {
            linearize_binary(var_factory, blocks, current, Op::Times, *left, *right)
        }
        prev::Expr::Divide { left, right } => {
            linearize_binary(var_factory, blocks, current, Op::Divide, *left, *right)
        }
        prev::Expr::If { cond, thn, els } => {
            let cond = match *cond {
                prev::Expr::Eq { left, right } => Condition::Cmp {
                    cmp: Cmp::Eq,
                    left: linearize_expr(var_factory, blocks, current, *left),
                    right: linearize_expr(var_factory, blocks, current, *right),
                },
                expr => Condition::Atm(linearize_expr(var_factory, blocks, current, expr)),
            };
            let var = var_factory.tmp();

            let mut thn_block = blocks.add_node(Block { stmts: Vec::new() });
            blocks.add_edge(*current, thn_block, Jmp::If(cond.clone()));
            linearize_branch(var_factory, blocks, &mut thn_block, var.clone(), *thn);

            let mut els_block = blocks.add_node(Block { stmts: Vec::new() });
            blocks.add_edge(*current, els_block, Jmp::Unless(cond));
            linearize_branch(var_factory, blocks, &mut els_block, var.clone(), *els);

            let after = blocks.add_node(Block { stmts: Vec::new() });
            blocks.add_edge(thn_block, after, Jmp::Unconditional);
            blocks.add_edge(els_block, after, Jmp::Unconditional);

            *current = after;
            Atom::Var(var)
        },
        prev::Expr::Eq { left, right } => {
            linearize_cmp(var_factory, blocks, current, Cmp::Eq, *left, *right)
        }
    }
}

fn linearize_binary(
    var_factory: &mut VarFactory,
    blocks: &mut Graph,
    current: &mut Index,
    op: Op,
    left: prev::Expr,
    right: prev::Expr,
) -> Atom {
    let left = linearize_expr(var_factory, blocks, current, left);
    let right = linearize_expr(var_factory, blocks, current, right);
    let var = var_factory.tmp();

    let stmts = &mut blocks.node_weight_mut(*current).unwrap().stmts;
    stmts.push(Statement::Assign {
        var: var.clone(),
        expr: Expr::Binary { left, right, op },
    });

    Atom::Var(var)
}

fn linearize_cmp(
    var_factory: &mut VarFactory,
    blocks: &mut Graph,
    current: &mut Index,
    cmp: Cmp,
    left: prev::Expr,
    right: prev::Expr,
) -> Atom {
    let left = linearize_expr(var_factory, blocks, current, left);
    let right = linearize_expr(var_factory, blocks, current, right);
    let var = var_factory.tmp();

    let stmts = &mut blocks.node_weight_mut(*current).unwrap().stmts;
    stmts.push(Statement::Assign {
        var: var.clone(),
        expr: Expr::Cmp { left, right, cmp },
    });

    Atom::Var(var)
}

fn linearize_branch(var_factory: &mut VarFactory, blocks: &mut Graph, current: &mut Index, var: Var, expr: prev::Expr) {
    let atm = linearize_expr(var_factory, blocks, current, expr);
    let stmts = &mut blocks.node_weight_mut(*current).unwrap().stmts;
    stmts.push(Statement::Assign { var, expr: Expr::Atom(atm) });
}

#[cfg(test)]
mod test {
    use petgraph::{graph::EdgeReference, visit::{EdgeRef, IntoEdgesDirected}};

    use super::*;

    // #[test]
    // fn assert_literal() {
    //     let def = prev::Definition::Test {
    //         name: "test".to_owned(),
    //         stmts: vec![prev::Statement::Expr (
    //             prev::Expr::LitBool(true),
    //         )],
    //     };

    //     let program = linearize(prev::Program {
    //         defs: vec![def],
    //         var_factory: VarFactory::new(),
    //     });
    //     let block = program.blocks[program.tests.first().unwrap().block].clone();

    //     assert_eq!(
    //         Block {
    //             stmts: vec![Statement::Assert {
    //                 atom: Atom::LitBool(true)
    //             }]
    //         },
    //         block
    //     );
    // }

    // #[test]
    // fn assert_eq_literal() {
    //     let def = prev::Definition::Test {
    //         name: "test".to_owned(),
    //         stmts: vec![prev::Statement::AssertEq {
    //             left: prev::Expr::LitInt(11),
    //             right: prev::Expr::LitInt(12),
    //         }],
    //     };

    //     let program = linearize(prev::Program {
    //         defs: vec![def],
    //         var_factory: VarFactory::new(),
    //     });
    //     let block = program.blocks[program.tests.first().unwrap().block].clone();

    //     assert_eq!(
    //         Block {
    //             stmts: vec![Statement::AssertEq {
    //                 left: Atom::LitInt(11),
    //                 right: Atom::LitInt(12)
    //             }]
    //         },
    //         block
    //     );
    // }

    #[test]
    fn command() {
        let def = prev::Definition::Test {
            name: "test".to_owned(),
            stmts: vec![prev::Statement::Command {
                text: "command text".to_owned(),
            }],
        };

        let program = linearize(prev::Program {
            defs: vec![def],
            var_factory: VarFactory::new(),
        });
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
        let def = prev::Definition::Test {
            name: "test".to_owned(),
            stmts: vec![prev::Statement::Expr(
                prev::Expr::Plus {
                    left: Box::new(prev::Expr::Times {
                        left: Box::new(prev::Expr::LitInt(1)),
                        right: Box::new(prev::Expr::Minus {
                            left: Box::new(prev::Expr::LitInt(2)),
                            right: Box::new(prev::Expr::LitInt(3)),
                        }),
                    }),
                    right: Box::new(prev::Expr::Divide {
                        left: Box::new(prev::Expr::LitInt(4)),
                        right: Box::new(prev::Expr::LitInt(5)),
                    }),
                },
            )],
        };

        let program = linearize(prev::Program {
            defs: vec![def],
            var_factory: VarFactory::new(),
        });
        let block = program.blocks[program.tests.first().unwrap().block].clone();
        let stmts = block.stmts;

        // tmp1 = (- 2 3)
        let tmp1 = if let Statement::Assign {
            var: tmp1,
            expr:
                Expr::Binary {
                    left,
                    right,
                    op: Op::Minus,
                },
        } = &stmts[0]
        {
            assert_eq!(left, &Atom::LitInt(2));
            assert_eq!(right, &Atom::LitInt(3));
            tmp1.clone()
        } else {
            panic!("Expected tmp1 = (- 2 3)");
        };

        // tmp2 = (* 1 tmp1)
        let tmp2 = if let Statement::Assign {
            var: tmp2,
            expr:
                Expr::Binary {
                    left,
                    right,
                    op: Op::Times,
                },
        } = &stmts[1]
        {
            assert_eq!(left, &Atom::LitInt(1));
            assert_eq!(right, &Atom::Var(tmp1));
            tmp2.clone()
        } else {
            panic!("Expected tmp2 = (* 1 tmp1)");
        };

        // tmp3 = (/ 4 5)
        let tmp3 = if let Statement::Assign {
            var: tmp3,
            expr:
                Expr::Binary {
                    left,
                    right,
                    op: Op::Divide,
                },
        } = &stmts[2]
        {
            assert_eq!(left, &Atom::LitInt(4));
            assert_eq!(right, &Atom::LitInt(5));
            tmp3.clone()
        } else {
            panic!("Expected tmp3 = (/ 4 5)");
        };

        // tmp4 = (+ tmp2 tmp3)
        let tmp4 = if let Statement::Assign {
            var: tmp4,
            expr:
                Expr::Binary {
                    left,
                    right,
                    op: Op::Plus,
                },
        } = &stmts[3]
        {
            assert_eq!(left, &Atom::Var(tmp2));
            assert_eq!(right, &Atom::Var(tmp3));
            tmp4.clone()
        } else {
            panic!("Expected tmp2 = (+ tmp2 tmp3)");
        };

        assert_eq!(stmts.len(), 4);
    }

    #[test]
    fn let_stmt() {
        let mut var_factory = VarFactory::new();

        // (let (x 2)) (+ x 1)
        let x = var_factory.named("x".to_owned());
        let def = prev::Definition::Test {
            name: "test".to_owned(),
            stmts: vec![
                prev::Statement::Let {
                    var: x.clone(),
                    expr: prev::Expr::LitInt(2),
                },
                prev::Statement::Expr (
                    prev::Expr::Plus {
                        left: Box::new(prev::Expr::Variable(x.clone())),
                        right: Box::new(prev::Expr::LitInt(1)),
                    },
                ),
            ],
        };

        let program = linearize(prev::Program {
            defs: vec![def],
            var_factory,
        });
        let block = program.blocks[program.tests.first().unwrap().block].clone();
        let stmts = block.stmts;

        // x = 2
        if let Statement::Assign {
            var,
            expr: Expr::Atom(Atom::LitInt(2)),
        } = &stmts[0]
        {
            assert_eq!(var, &x);
        } else {
            panic!("Expected x = 2");
        };

        // tmp1 = (+ x 1)
        if let Statement::Assign {
            var: _,
            expr:
                Expr::Binary {
                    left,
                    right,
                    op: Op::Plus,
                },
        } = &stmts[1]
        {
            assert_eq!(left, &Atom::Var(x));
            assert_eq!(right, &Atom::LitInt(1));
        } else {
            panic!("Expected tmp1 = (+ x 1)");
        };

        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn if_expr() {
        // (if true false true)
        let def = prev::Definition::Test {
            name: "test".to_owned(),
            stmts: vec![
                prev::Statement::Expr(prev::Expr::If { cond: Box::new(prev::Expr::LitBool(true)), thn: Box::new(prev::Expr::LitBool(false)), els: Box::new(prev::Expr::LitBool(true)) })
            ],
        };

        let program = linearize(prev::Program {
            defs: vec![def],
            var_factory: VarFactory::new(),
        });
        let test = program.tests.first().unwrap().block;
        let block = program.blocks[test].clone();
        let stmts = block.stmts;
        let edges: Vec<EdgeReference<Jmp>> = program.blocks.edges_directed(test, petgraph::Direction::Outgoing).collect();

        assert!(stmts.is_empty());

        assert_eq!(edges[1].weight().clone(), Jmp::If(Condition::Atm(Atom::LitBool(true))));
        assert_eq!(edges[0].weight().clone(), Jmp::Unless(Condition::Atm(Atom::LitBool(true))));

        let thn_stmts = program.blocks[edges[1].target()].stmts.clone();
        let Statement::Assign { var: tmp, expr: Expr::Atom(Atom::LitBool(false)) } = thn_stmts[0].clone() else {
            panic!();
        };
        let after_thn: EdgeReference<Jmp> = program.blocks.edges_directed(edges[0].target(), petgraph::Direction::Outgoing).next().unwrap();

        let els_stmts = program.blocks[edges[0].target()].stmts.clone();
        let Statement::Assign { var: tmp_prime, expr: Expr::Atom(Atom::LitBool(true)) } = els_stmts[0].clone() else {
            panic!();
        };
        let after_els: EdgeReference<Jmp> = program.blocks.edges_directed(edges[1].target(), petgraph::Direction::Outgoing).next().unwrap();

        assert_eq!(tmp, tmp_prime);
        assert_eq!(after_thn.target(), after_els.target());
        assert_eq!(after_thn.weight().clone(), Jmp::Unconditional);

        let after_stmts = program.blocks[after_thn.target()].stmts.clone();
        assert_eq!(after_stmts.len(), 0);
    }
}

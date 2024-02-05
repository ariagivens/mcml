use crate::uniquify as prev;
use crate::var::{Var, VarFactory};

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
    Assign { var: Var, expr: Expr },
    Assert { atom: Atom },
    AssertEq { left: Atom, right: Atom },
    Command { text: String },
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

    Program { blocks, tests }
}

fn linearize_stmts(
    var_factory: &mut VarFactory,
    blocks: &mut Graph,
    stmts: Vec<prev::Statement>,
) -> Index {
    let mut new_stmts = Vec::new();

    for stmt in stmts {
        new_stmts.extend(linearize_stmt(var_factory, stmt));
    }

    let block = Block { stmts: new_stmts };

    blocks.add_node(block)
}

fn linearize_stmt(var_factory: &mut VarFactory, stmt: prev::Statement) -> Vec<Statement> {
    let stmts = match stmt {
        prev::Statement::Assert { expr } => {
            let (atom, mut stmts) = linearize_expr(var_factory, expr);
            stmts.push(Statement::Assert { atom });
            stmts
        }
        prev::Statement::AssertEq { left, right } => {
            let (left, mut stmts) = linearize_expr(var_factory, left);
            let (right, right_stmts) = linearize_expr(var_factory, right);
            stmts.extend(right_stmts);
            stmts.push(Statement::AssertEq { left, right });
            stmts
        }
        prev::Statement::Command { text } => {
            vec![Statement::Command { text }]
        }
        prev::Statement::Let { var, expr } => {
            let (atom, mut stmts) = linearize_expr(var_factory, expr);
            stmts.push(Statement::Assign {
                var,
                expr: Expr::Atom(atom),
            });
            stmts
        }
    };

    stmts
}

fn linearize_expr(var_factory: &mut VarFactory, expr: prev::Expr) -> (Atom, Vec<Statement>) {
    match expr {
        prev::Expr::LitBool(b) => (Atom::LitBool(b), vec![]),
        prev::Expr::LitInt(i) => (Atom::LitInt(i), vec![]),
        prev::Expr::Variable(x) => (Atom::Var(x), vec![]),
        prev::Expr::Plus { left, right } => {
            linearize_binary(var_factory, Expr::Plus, *left, *right)
        }
        prev::Expr::Minus { left, right } => {
            linearize_binary(var_factory, Expr::Minus, *left, *right)
        }
        prev::Expr::Times { left, right } => {
            linearize_binary(var_factory, Expr::Times, *left, *right)
        }
        prev::Expr::Divide { left, right } => {
            linearize_binary(var_factory, Expr::Divide, *left, *right)
        }
    }
}

fn linearize_binary(
    var_factory: &mut VarFactory,
    op: impl Fn(Binary) -> Expr,
    left: prev::Expr,
    right: prev::Expr,
) -> (Atom, Vec<Statement>) {
    let (left, mut stmts) = linearize_expr(var_factory, left);
    let (right, right_stmts) = linearize_expr(var_factory, right);
    stmts.extend(right_stmts);

    let var = var_factory.tmp();
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
        let def = prev::Definition::Test {
            name: "test".to_owned(),
            stmts: vec![prev::Statement::Assert {
                expr: prev::Expr::LitBool(true),
            }],
        };

        let program = linearize(prev::Program {
            defs: vec![def],
            var_factory: VarFactory::new(),
        });
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
        let def = prev::Definition::Test {
            name: "test".to_owned(),
            stmts: vec![prev::Statement::AssertEq {
                left: prev::Expr::LitInt(11),
                right: prev::Expr::LitInt(12),
            }],
        };

        let program = linearize(prev::Program {
            defs: vec![def],
            var_factory: VarFactory::new(),
        });
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
            stmts: vec![prev::Statement::Assert {
                expr: prev::Expr::Plus {
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
            }],
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
            expr: Expr::Minus(Binary { left, right }),
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
            expr: Expr::Times(Binary { left, right }),
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
            expr: Expr::Divide(Binary { left, right }),
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
            expr: Expr::Plus(Binary { left, right }),
        } = &stmts[3]
        {
            assert_eq!(left, &Atom::Var(tmp2));
            assert_eq!(right, &Atom::Var(tmp3));
            tmp4.clone()
        } else {
            panic!("Expected tmp2 = (+ tmp2 tmp3)");
        };

        // assert tmp4
        assert_eq!(
            Statement::Assert {
                atom: Atom::Var(tmp4)
            },
            stmts[4]
        );
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
                prev::Statement::AssertEq {
                    left: prev::Expr::Plus {
                        left: Box::new(prev::Expr::Variable(x.clone())),
                        right: Box::new(prev::Expr::LitInt(1)),
                    },
                    right: prev::Expr::LitInt(3),
                },
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
        let tmp1 = if let Statement::Assign {
            var: tmp1,
            expr: Expr::Plus(Binary { left, right }),
        } = &stmts[1]
        {
            assert_eq!(left, &Atom::Var(x));
            assert_eq!(right, &Atom::LitInt(1));
            tmp1.clone()
        } else {
            panic!("Expected tmp1 = (+ x 1)");
        };

        // asserteq tmp1 3
        assert_eq!(
            Statement::AssertEq {
                left: Atom::Var(tmp1),
                right: Atom::LitInt(3)
            },
            stmts[2]
        );
    }
}

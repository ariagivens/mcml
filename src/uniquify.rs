use std::collections::HashMap;

use crate::parse as prev;
use crate::var::{Var, VarFactory};

pub struct Program {
    pub defs: Vec<Definition>,
    pub var_factory: VarFactory,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Definition {
    Test { name: String, stmts: Vec<Statement> },
}

#[derive(PartialEq, Eq, Debug)]
pub enum Statement {
    Assert { expr: Expr },
    AssertEq { left: Expr, right: Expr },
    Command { text: String },
    Let { var: Var, expr: Expr },
}

#[derive(PartialEq, Eq, Debug)]
pub enum Expr {
    LitBool(bool),
    LitInt(i64),
    Variable(Var),
    Plus {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Minus {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Times {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Divide {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        thn: Box<Expr>,
        els: Box<Expr>,
    },
    Eq {
        left: Box<Expr>,
        right: Box<Expr>,
    }
}

type Env = HashMap<String, Var>;

pub fn uniquify(defs: Vec<prev::Definition>) -> Program {
    let mut new_defs = Vec::new();
    let mut var_factory = VarFactory::new();

    for def in defs {
        let prev::Definition::Test { name, stmts } = def;
        new_defs.push(uniquify_test(&mut var_factory, name, stmts));
    }

    Program {
        defs: new_defs,
        var_factory,
    }
}

fn uniquify_test(
    var_factory: &mut VarFactory,
    name: String,
    stmts: Vec<prev::Statement>,
) -> Definition {
    let mut env = Env::new();
    let mut new_stmts = Vec::new();

    for stmt in stmts {
        new_stmts.push(match stmt {
            prev::Statement::Assert { expr } => Statement::Assert {
                expr: uniquify_expr(&env, expr),
            },
            prev::Statement::AssertEq { left, right } => Statement::AssertEq {
                left: uniquify_expr(&env, left),
                right: uniquify_expr(&env, right),
            },
            prev::Statement::Command { text } => Statement::Command { text },
            prev::Statement::Let {
                variable_name,
                expr,
            } => {
                let expr = uniquify_expr(&env, expr);
                let var = var_factory.named(variable_name.clone());
                env.insert(variable_name, var.clone());
                Statement::Let { var, expr }
            }
        });
    }

    Definition::Test {
        name,
        stmts: new_stmts,
    }
}

fn uniquify_expr(env: &Env, expr: prev::Expr) -> Expr {
    match expr {
        prev::Expr::LitBool(b) => Expr::LitBool(b),
        prev::Expr::LitInt(i) => Expr::LitInt(i),
        prev::Expr::Variable(name) => Expr::Variable(env[&name].clone()),
        prev::Expr::Plus { left, right } => Expr::Plus {
            left: Box::new(uniquify_expr(env, *left)),
            right: Box::new(uniquify_expr(env, *right)),
        },
        prev::Expr::Minus { left, right } => Expr::Minus {
            left: Box::new(uniquify_expr(env, *left)),
            right: Box::new(uniquify_expr(env, *right)),
        },
        prev::Expr::Times { left, right } => Expr::Times {
            left: Box::new(uniquify_expr(env, *left)),
            right: Box::new(uniquify_expr(env, *right)),
        },
        prev::Expr::Divide { left, right } => Expr::Divide {
            left: Box::new(uniquify_expr(env, *left)),
            right: Box::new(uniquify_expr(env, *right)),
        },
        prev::Expr::If { cond, thn, els } => Expr::If {
            cond: Box::new(uniquify_expr(env, *cond)),
            thn: Box::new(uniquify_expr(env, *thn)),
            els: Box::new(uniquify_expr(env, *els)),
        },
        prev::Expr::Eq { left, right } => Expr::Eq {
            left: Box::new(uniquify_expr(env, *left)),
            right: Box::new(uniquify_expr(env, *right)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse as prev;

    #[test]
    fn simple() {
        let defs = vec![prev::Definition::Test {
            name: "test".to_owned(),
            stmts: vec![
                prev::Statement::Let {
                    variable_name: "x".to_owned(),
                    expr: prev::Expr::LitBool(true),
                },
                prev::Statement::Assert {
                    expr: prev::Expr::Variable("x".to_owned()),
                },
            ],
        }];

        let program = uniquify(defs);
        let Definition::Test { stmts, .. } = &program.defs[0];

        let x1 = if let Statement::Let { var, expr: _ } = &stmts[0] {
            var
        } else {
            panic!("Expected let statement");
        };

        if let Statement::Assert {
            expr: Expr::Variable(x2),
        } = &stmts[1]
        {
            assert_eq!(x1, x2);
        }
    }

    #[test]
    fn shadowing() {
        let defs = vec![prev::Definition::Test {
            name: "test".to_owned(),
            stmts: vec![
                prev::Statement::Let {
                    variable_name: "x".to_owned(),
                    expr: prev::Expr::LitBool(true),
                },
                prev::Statement::Assert {
                    expr: prev::Expr::Variable("x".to_owned()),
                },
                prev::Statement::Let {
                    variable_name: "x".to_owned(),
                    expr: prev::Expr::LitBool(true),
                },
                prev::Statement::Assert {
                    expr: prev::Expr::Variable("x".to_owned()),
                },
            ],
        }];

        let program = uniquify(defs);
        let Definition::Test { stmts, .. } = &program.defs[0];

        let x1 = if let Statement::Let { var, expr: _ } = &stmts[0] {
            var
        } else {
            panic!("Expected let statement");
        };

        if let Statement::Assert {
            expr: Expr::Variable(x2),
        } = &stmts[1]
        {
            assert_eq!(x1, x2);
        }

        let x3 = if let Statement::Let { var, expr: _ } = &stmts[2] {
            var
        } else {
            panic!("Expected let statement");
        };

        if let Statement::Assert {
            expr: Expr::Variable(x4),
        } = &stmts[3]
        {
            assert_eq!(x3, x4);
            assert_ne!(x1, x3);
        }
    }
}

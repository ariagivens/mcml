use crate::uniquify as prev;
use crate::var::{Var, VarFactory};

pub struct Program {
    pub defs: Vec<Definition>,
    pub var_factory: VarFactory,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Definition {
    Test { name: String, stmts: Vec<Statement> },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Statement {
    Expr(Expr),
    Command { text: String },
    Let { var: Var, expr: Expr },
    TellOk { test_name: String },
    TellNotOk { test_name: String },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expr {
    LitUnit,
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
    },
    Bundle {
        stmts: Vec<Statement>,
        expr: Box<Expr>,
    },
}

pub fn desugar_asserts(program: prev::Program) -> Program {
    let defs = program.defs.into_iter().map(desugar_asserts_def).collect();

    Program {
        defs,
        var_factory: program.var_factory,
    }
}

fn desugar_asserts_def(def: prev::Definition) -> Definition {
    match def {
        prev::Definition::Test { name, stmts } => {
            // Probably more elegant as an rfold.
            let mut new_stmts = vec![Statement::TellOk {
                test_name: name.clone(),
            }];
            for stmt in stmts.into_iter().rev() {
                new_stmts = desugar_asserts_stmts(&name, stmt, new_stmts);
            }

            Definition::Test {
                name,
                stmts: new_stmts,
            }
        }
    }
}

fn desugar_asserts_stmts(
    test_name: &str,
    stmt: prev::Statement,
    mut continuation: Vec<Statement>,
) -> Vec<Statement> {
    let mut stmts = Vec::new();

    match stmt {
        prev::Statement::Assert { expr } => {
            stmts.push(Statement::Expr(Expr::If {
                cond: Box::new(desugar_asserts_expr(expr)),
                thn: Box::new(Expr::Bundle {
                    stmts: continuation,
                    expr: Box::new(Expr::LitUnit),
                }),
                els: Box::new(Expr::Bundle {
                    stmts: vec![Statement::TellNotOk {
                        test_name: test_name.to_string(),
                    }],
                    expr: Box::new(Expr::LitUnit),
                }),
            }));
        }
        prev::Statement::AssertEq { left, right } => {
            stmts.push(Statement::Expr(Expr::If {
                cond: Box::new(Expr::Eq {
                    left: Box::new(desugar_asserts_expr(left)),
                    right: Box::new(desugar_asserts_expr(right)),
                }),
                thn: Box::new(Expr::Bundle {
                    stmts: continuation,
                    expr: Box::new(Expr::LitUnit),
                }),
                els: Box::new(Expr::Bundle {
                    stmts: vec![Statement::TellNotOk {
                        test_name: test_name.to_string(),
                    }],
                    expr: Box::new(Expr::LitUnit),
                }),
            }));
        },
        prev::Statement::Command { text } => {
            stmts.push(Statement::Command { text });
            stmts.extend(continuation);
        }
        prev::Statement::Let { var, expr } => {
            stmts.push(Statement::Let {
                var,
                expr: desugar_asserts_expr(expr),
            });
            stmts.extend(continuation);
        }
    }

    stmts
}

fn desugar_asserts_expr(expr: prev::Expr) -> Expr {
    match expr {
        prev::Expr::LitBool(b) => Expr::LitBool(b),
        prev::Expr::LitInt(i) => Expr::LitInt(i),
        prev::Expr::Variable(var) => Expr::Variable(var),
        prev::Expr::Plus { left, right } => Expr::Plus {
            left: Box::new(desugar_asserts_expr(*left)),
            right: Box::new(desugar_asserts_expr(*right)),
        },
        prev::Expr::Minus { left, right } => Expr::Minus {
            left: Box::new(desugar_asserts_expr(*left)),
            right: Box::new(desugar_asserts_expr(*right)),
        },
        prev::Expr::Times { left, right } => Expr::Times {
            left: Box::new(desugar_asserts_expr(*left)),
            right: Box::new(desugar_asserts_expr(*right)),
        },
        prev::Expr::Divide { left, right } => Expr::Divide {
            left: Box::new(desugar_asserts_expr(*left)),
            right: Box::new(desugar_asserts_expr(*right)),
        },
        prev::Expr::If { cond, thn, els } => Expr::If {
            cond: Box::new(desugar_asserts_expr(*cond)),
            thn: Box::new(desugar_asserts_expr(*thn)),
            els: Box::new(desugar_asserts_expr(*els)),
        },
        prev::Expr::Eq { left, right } => Expr::Eq {
            left: Box::new(desugar_asserts_expr(*left)),
            right: Box::new(desugar_asserts_expr(*right)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_asserts() {
        let mut var_factory = VarFactory::new();
        let x = var_factory.named("x".to_owned());

        let program = prev::Program {
            defs: vec![prev::Definition::Test {
                name: "test".to_owned(),
                stmts: vec![
                    prev::Statement::Let {
                        var: x.clone(),
                        expr: prev::Expr::LitBool(false),
                    },
                    prev::Statement::Command {
                        text: "text".to_owned(),
                    },
                ],
            }],
            var_factory: VarFactory::new(),
        };

        let program = desugar_asserts(program);
        let Definition::Test { stmts, .. } = &program.defs[0];

        assert_eq!(
            stmts[0],
            Statement::Let {
                var: x,
                expr: Expr::LitBool(false)
            }
        );
        assert_eq!(
            stmts[1],
            Statement::Command {
                text: "text".to_owned()
            }
        );
        assert_eq!(
            stmts[2],
            Statement::TellOk {
                test_name: "test".to_owned()
            }
        );
    }

    #[test]
    fn multi_asserts() {
        let mut var_factory = VarFactory::new();
        let x = var_factory.named("x".to_owned());
        let y = var_factory.named("y".to_owned());
        let z = var_factory.named("z".to_owned());

        let program = prev::Program {
            defs: vec![prev::Definition::Test {
                name: "test".to_owned(),
                stmts: vec![
                    prev::Statement::Let {
                        var: x.clone(),
                        expr: prev::Expr::LitBool(false),
                    },
                    prev::Statement::Assert { expr: prev::Expr::LitBool(true) },
                    prev::Statement::Let {
                        var: y.clone(),
                        expr: prev::Expr::LitBool(false),
                    },
                    prev::Statement::Assert { expr: prev::Expr::LitBool(false) },
                    prev::Statement::Let {
                        var: z.clone(),
                        expr: prev::Expr::LitBool(false),
                    },

                ],
            }],
            var_factory: VarFactory::new(),
        };

        let program = desugar_asserts(program);
        let Definition::Test { stmts, .. } = &program.defs[0];

        assert_eq!(
            stmts[0],
            Statement::Let {
                var: x,
                expr: Expr::LitBool(false)
            }
        );

        let stmts = if let Statement::Expr(Expr::If { cond, thn, els}) = stmts[1].clone() {
            assert_eq!(*cond, Expr::LitBool(true));
            assert_eq!(*els, Expr::Bundle { stmts: vec![ Statement::TellNotOk { test_name: "test".to_owned() }], expr: Box::new(Expr::LitUnit) });
            if let Expr::Bundle { stmts, expr } = *thn {
                assert_eq!(*expr, Expr::LitUnit);
                stmts
            } else {
                panic!()
            }
        } else {
            panic!()
        };

        assert_eq!(
            stmts[0],
            Statement::Let {
                var: y,
                expr: Expr::LitBool(false)
            }
        );

        let stmts = if let Statement::Expr(Expr::If { cond, thn, els }) = stmts[1].clone() {
            assert_eq!(*cond, Expr::LitBool(false));
            assert_eq!(*els, Expr::Bundle { stmts: vec![ Statement::TellNotOk { test_name: "test".to_owned() }], expr: Box::new(Expr::LitUnit) });
            if let Expr::Bundle { stmts, expr } = *thn {
                assert_eq!(*expr, Expr::LitUnit);
                stmts
            } else {
                panic!()
            }
        } else {
            panic!()
        };

        assert_eq!(
            stmts[0],
            Statement::Let {
                var: z,
                expr: Expr::LitBool(false)
            }
        );
        assert_eq!(
            stmts[1],
            Statement::TellOk { test_name: "test".to_owned() }
        );
    }
}

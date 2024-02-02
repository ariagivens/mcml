use crate::lex::Token;
use anyhow::{anyhow, Result};
use std::collections::VecDeque;

#[derive(PartialEq, Eq, Debug)]
pub enum Definition {
    Test { name: String, stmts: Vec<Statement> },
}

#[derive(PartialEq, Eq, Debug)]
pub enum Statement {
    Assert { expr: Expr },
    AssertEq { left: Expr, right: Expr },
    Command { text: String },
    Let { variable_name: String, expr: Expr },
}

#[derive(PartialEq, Eq, Debug)]
pub enum Expr {
    LitBool(bool),
    LitInt(i64),
    Variable(String),
    Plus { left: Box<Expr>, right: Box<Expr> },
    Minus { left: Box<Expr>, right: Box<Expr> },
    Times { left: Box<Expr>, right: Box<Expr> },
    Divide { left: Box<Expr>, right: Box<Expr> },
}

struct Tokens {
    inner: VecDeque<Token>,
}

impl Tokens {
    fn new(inner: Vec<Token>) -> Self {
        Tokens {
            inner: VecDeque::from(inner),
        }
    }

    fn require(&mut self, token: Token) -> Result<()> {
        match self.inner.pop_front() {
            Some(t) if t == token => Ok(()),
            Some(t) => Err(anyhow!("Expected {} but saw {}", token, t)),
            None => Err(anyhow!("Expected {} but ran out of tokens", token)),
        }
    }

    fn next(&mut self) -> Result<Token> {
        match self.inner.pop_front() {
            Some(t) => Ok(t),
            None => Err(anyhow!("Ran out of tokens")),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Definition>> {
    let mut tokens = Tokens::new(tokens);
    let mut defs = Vec::new();
    while let Ok(token) = tokens.next() {
        if token == Token::LeftParen {
            defs.push(parse_definition(&mut tokens)?);
        } else {
            return Err(anyhow!("Unexpected token: {}", token));
        }
    }
    Ok(defs)
}

fn parse_definition(mut tokens: &mut Tokens) -> Result<Definition> {
    match tokens.next()? {
        Token::Test => parse_test(&mut tokens),
        _ => Err(anyhow!("Unexpected thingy")),
    }
}

fn parse_test(tokens: &mut Tokens) -> Result<Definition> {
    if let Token::String(name) = tokens.next()? {
        let mut stmts = Vec::new();
        loop {
            match tokens.next()? {
                Token::LeftParen => stmts.push(parse_stmt(tokens)?),
                Token::RightParen => return Ok(Definition::Test { name, stmts }),
                x => return Err(anyhow!("Expected statement saw {}", x)),
            }
        }
    } else {
        Err(anyhow!("Expected test to have name"))
    }
}

fn parse_stmt(tokens: &mut Tokens) -> Result<Statement> {
    let stmt = match tokens.next()? {
        Token::Assert => Ok(Statement::Assert {
            expr: parse_expr(tokens)?,
        }),
        Token::AssertEq => Ok(Statement::AssertEq {
            left: parse_expr(tokens)?,
            right: parse_expr(tokens)?,
        }),
        Token::Slash => {
            if let Token::String(text) = tokens.next()? {
                Ok(Statement::Command { text })
            } else {
                Err(anyhow!("Expected a string to follow /"))
            }
        }
        Token::Let => {
            tokens.require(Token::LeftParen)?;
            if let Token::Ident(variable_name) = tokens.next()? {
                let expr = parse_expr(tokens)?;
                tokens.require(Token::RightParen)?;
                Ok(Statement::Let {
                    variable_name,
                    expr,
                })
            } else {
                Err(anyhow!("Expected variable identifier"))
            }
        }
        _ => Err(anyhow!("Expected a statement")),
    }?;
    tokens.require(Token::RightParen)?;
    Ok(stmt)
}

fn parse_expr(tokens: &mut Tokens) -> Result<Expr> {
    match tokens.next()? {
        Token::Boolean(b) => Ok(Expr::LitBool(b)),
        Token::Int(i) => Ok(Expr::LitInt(i)),
        Token::Ident(x) => Ok(Expr::Variable(x)),
        Token::LeftParen => parse_arithmetic(tokens),
        _ => Err(anyhow!("Expected an expression")),
    }
}

fn parse_arithmetic(tokens: &mut Tokens) -> Result<Expr> {
    let expr = match tokens.next()? {
        Token::Plus => Expr::Plus {
            left: Box::new(parse_expr(tokens)?),
            right: Box::new(parse_expr(tokens)?),
        },
        Token::Dash => Expr::Minus {
            left: Box::new(parse_expr(tokens)?),
            right: Box::new(parse_expr(tokens)?),
        },
        Token::Star => Expr::Times {
            left: Box::new(parse_expr(tokens)?),
            right: Box::new(parse_expr(tokens)?),
        },
        Token::Slash => Expr::Divide {
            left: Box::new(parse_expr(tokens)?),
            right: Box::new(parse_expr(tokens)?),
        },
        _ => return Err(anyhow!("Expected arithmetic expression")),
    };
    tokens.require(Token::RightParen)?;
    Ok(expr)
}

#[cfg(test)]
mod test {
    use super::Token::*;
    use super::*;

    #[test]
    fn assert_bool() -> Result<()> {
        let tokens = vec![
            LeftParen,
            Test,
            String(r#"test"#.to_owned()),
            LeftParen,
            Assert,
            Boolean(true),
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test".to_owned(),
                stmts: vec![Statement::Assert {
                    expr: Expr::LitBool(true)
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn command_literal() -> Result<()> {
        let tokens = vec![
            LeftParen,
            Test,
            String("test 2".to_owned()),
            LeftParen,
            Slash,
            String("cmd text".to_owned()),
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test 2".to_owned(),
                stmts: vec![Statement::Command {
                    text: "cmd text".to_owned()
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn asserteq_ints() -> Result<()> {
        let tokens = vec![
            LeftParen,
            Test,
            String("test 3".to_owned()),
            LeftParen,
            AssertEq,
            Int(5),
            Int(-5),
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test 3".to_owned(),
                stmts: vec![Statement::AssertEq {
                    left: Expr::LitInt(5),
                    right: Expr::LitInt(-5)
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn multitest() -> Result<()> {
        let tokens = vec![
            LeftParen,
            Test,
            String(r#"test 1"#.to_owned()),
            LeftParen,
            Assert,
            Boolean(true),
            RightParen,
            RightParen,
            LeftParen,
            Test,
            String(r#"test 2"#.to_owned()),
            LeftParen,
            Assert,
            Boolean(true),
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![
                Definition::Test {
                    name: "test 1".to_owned(),
                    stmts: vec![Statement::Assert {
                        expr: Expr::LitBool(true)
                    }]
                },
                Definition::Test {
                    name: "test 2".to_owned(),
                    stmts: vec![Statement::Assert {
                        expr: Expr::LitBool(true)
                    }]
                }
            ],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn addition() -> Result<()> {
        let tokens = vec![
            LeftParen,
            Test,
            String(r#"test"#.to_owned()),
            LeftParen,
            Assert,
            LeftParen,
            Plus,
            Int(1),
            Int(1),
            RightParen,
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test".to_owned(),
                stmts: vec![Statement::Assert {
                    expr: Expr::Plus {
                        left: Box::new(Expr::LitInt(1)),
                        right: Box::new(Expr::LitInt(1))
                    }
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn subtraction() -> Result<()> {
        let tokens = vec![
            LeftParen,
            Test,
            String(r#"test"#.to_owned()),
            LeftParen,
            Assert,
            LeftParen,
            Dash,
            Int(1),
            Int(1),
            RightParen,
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test".to_owned(),
                stmts: vec![Statement::Assert {
                    expr: Expr::Minus {
                        left: Box::new(Expr::LitInt(1)),
                        right: Box::new(Expr::LitInt(1))
                    }
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn multiplication() -> Result<()> {
        let tokens = vec![
            LeftParen,
            Test,
            String(r#"test"#.to_owned()),
            LeftParen,
            Assert,
            LeftParen,
            Star,
            Int(1),
            Int(1),
            RightParen,
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test".to_owned(),
                stmts: vec![Statement::Assert {
                    expr: Expr::Times {
                        left: Box::new(Expr::LitInt(1)),
                        right: Box::new(Expr::LitInt(1))
                    }
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn division() -> Result<()> {
        let tokens = vec![
            LeftParen,
            Test,
            String(r#"test"#.to_owned()),
            LeftParen,
            Assert,
            LeftParen,
            Slash,
            Int(1),
            Int(1),
            RightParen,
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test".to_owned(),
                stmts: vec![Statement::Assert {
                    expr: Expr::Divide {
                        left: Box::new(Expr::LitInt(1)),
                        right: Box::new(Expr::LitInt(1))
                    }
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn nested_arithmetic() -> Result<()> {
        let tokens = vec![
            LeftParen,
            Test,
            String(r#"test"#.to_owned()),
            LeftParen,
            Assert,
            LeftParen,
            Plus,
            Int(1),
            LeftParen,
            Star,
            Int(1),
            Int(1),
            RightParen,
            RightParen,
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test".to_owned(),
                stmts: vec![Statement::Assert {
                    expr: Expr::Plus {
                        left: Box::new(Expr::LitInt(1)),
                        right: Box::new(Expr::Times {
                            left: Box::new(Expr::LitInt(1)),
                            right: Box::new(Expr::LitInt(1))
                        })
                    }
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn let_statement() -> Result<()> {
        // (test "test" (let (x 1)))
        let tokens = vec![
            LeftParen,
            Test,
            String(r#"test"#.to_owned()),
            LeftParen,
            Let,
            LeftParen,
            Ident("x".to_owned()),
            Int(1),
            RightParen,
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test".to_owned(),
                stmts: vec![Statement::Let {
                    variable_name: "x".to_owned(),
                    expr: Expr::LitInt(1)
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn variable_usage() -> Result<()> {
        // (test "test" x)
        let tokens = vec![
            LeftParen,
            Test,
            String(r#"test"#.to_owned()),
            LeftParen,
            Assert,
            Ident("x".to_owned()),
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test".to_owned(),
                stmts: vec![Statement::Assert {
                    expr: Expr::Variable("x".to_owned())
                }]
            }],
            parse(tokens)?
        );
        Ok(())
    }

    #[test]
    fn multiple_statements() -> Result<()> {
        // (test "test" (assert true) (assert false))
        let tokens = vec![
            LeftParen,
            Test,
            String("test".to_owned()),
            LeftParen,
            Assert,
            Boolean(true),
            RightParen,
            LeftParen,
            Assert,
            Boolean(false),
            RightParen,
            RightParen,
        ];
        assert_eq!(
            vec![Definition::Test {
                name: "test".to_owned(),
                stmts: vec![
                    Statement::Assert {
                        expr: Expr::LitBool(true)
                    },
                    Statement::Assert {
                        expr: Expr::LitBool(false)
                    }
                ]
            }],
            parse(tokens)?
        );

        Ok(())
    }
}

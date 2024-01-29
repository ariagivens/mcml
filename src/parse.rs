use std::collections::VecDeque;
use crate::lex::Token;
use anyhow::{Result, anyhow};

#[derive(PartialEq, Eq, Debug)]
pub enum ASTNode {
    Test { name: String, stmt: Statement },
}

#[derive(PartialEq, Eq, Debug)]
pub enum Statement {
    Assert { expr: Expr },
    Command { text: String }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Expr {
    LitBool(bool),
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

pub fn parse(tokens: Vec<Token>) -> Result<ASTNode> {
    let mut tokens = Tokens::new(tokens);
    tokens.require(Token::LeftParen)?;
    let node = match tokens.next()? {
        Token::Test => parse_test(&mut tokens),
        _ => Err(anyhow!("Unexpected thingy")),
    }?;
    tokens.require(Token::RightParen)?;
    Ok(node)
}

fn parse_test(tokens: &mut Tokens) -> Result<ASTNode> {
    if let Token::String(name) = tokens.next()? {
        tokens.require(Token::LeftParen)?;
        let stmt = parse_stmt(tokens)?;
        Ok(ASTNode::Test { name, stmt })
    } else {
        Err(anyhow!("Expected test to have name"))
    }
}

fn parse_stmt(tokens: &mut Tokens) -> Result<Statement> {
    let stmt = match tokens.next()? {
        Token::Assert => Ok(Statement::Assert {
            expr: parse_expr(tokens)?,
        }),
        Token::Slash => {
            if let Token::String(text) = tokens.next()? {
                Ok(Statement::Command { text })
            } else {
                Err(anyhow!("Expected a string to follow /"))
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
        _ => Err(anyhow!("Expected an expression")),
    }
}

#[test]
fn test_parse() -> Result<()> {
    use Token::*;
    let tokens = vec![
        LeftParen,
        Test,
        String(r#"test 1"#.to_owned()),
        LeftParen,
        Assert,
        Boolean(true),
        RightParen,
        RightParen,
    ];
    assert_eq!(
        ASTNode::Test {
            name: "test 1".to_owned(),
            stmt: Statement::Assert {
                expr: Expr::LitBool(true)
            }
        },
        parse(tokens)?
    );
    Ok(())
}

#[test]
fn test_parse_command() -> Result<()> {
    use Token::*;
    let tokens = vec![
        LeftParen, Test, String("test 2".to_owned()), LeftParen, Slash, String("cmd text".to_owned()), RightParen, RightParen
    ];
    assert_eq!(
        ASTNode::Test {
            name: "test 2".to_owned(),
            stmt: Statement::Command { text: "cmd text".to_owned() }
        },
        parse(tokens)?
    );
    
    Ok(())
}
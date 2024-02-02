use std::{fmt::Display, iter::Peekable};

use crate::utility::escape;
use anyhow::{anyhow, Result};

#[derive(PartialEq, Eq, Debug)]
pub enum Token {
    LeftParen,
    RightParen,
    Plus,
    Dash,
    Star,
    Slash,
    Ident(String),
    Test,
    Assert,
    AssertEq,
    Let,
    Boolean(bool),
    Int(i64),
    String(String),
}

impl Token {
    fn ident(s: String) -> Token {
        if &s == "test" {
            Token::Test
        } else if &s == "assert" {
            Token::Assert
        } else if &s == "asserteq" {
            Token::AssertEq
        } else if &s == "let" {
            Token::Let
        } else if &s == "true" {
            Token::Boolean(true)
        } else if &s == "false" {
            Token::Boolean(false)
        } else {
            Token::Ident(s)
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Plus => write!(f, "+"),
            Token::Dash => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Ident(i) => write!(f, "{}", i),
            Token::Test => write!(f, "test"),
            Token::Assert => write!(f, "assert"),
            Token::AssertEq => write!(f, "asserteq"),
            Token::Let => write!(f, "let"),
            Token::Boolean(b) => {
                if *b {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Token::Int(i) => write!(f, "{}", i),
            Token::String(s) => write!(f, "{}", escape(s)),
        }
    }
}

struct Characters {
    data: Vec<char>,
    counter: usize,
}

impl Characters {
    fn new(string: String) -> Self {
        Characters {
            data: string.chars().collect(),
            counter: 0,
        }
    }
}

impl Iterator for Characters {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.counter += 1;
        self.data.get(self.counter - 1).copied()
    }
}

pub fn lex(source: &str) -> Result<Vec<Token>> {
    let mut cs = Characters::new(source.to_owned()).peekable();
    let mut tokens = Vec::new();

    while let Some(c) = cs.next() {
        if c == '(' {
            tokens.push(Token::LeftParen);
        } else if c == ')' {
            tokens.push(Token::RightParen);
        } else if c == '+' {
            tokens.push(Token::Plus);
        } else if c == '*' {
            tokens.push(Token::Star);
        } else if c == '/' {
            tokens.push(Token::Slash);
        } else if c.is_alphabetic() {
            let mut s = String::from(c);

            while let Some(p) = cs.peek() {
                if p.is_alphanumeric() {
                    s.push(cs.next().unwrap())
                } else {
                    break;
                }
            }

            tokens.push(Token::ident(s))
        } else if c == '"' {
            let mut s = String::new();
            while let Some(c) = cs.next() {
                if c == '"' {
                    tokens.push(Token::String(s.clone()));
                    break;
                } else if c == '\\' {
                    match cs.next() {
                        Some('"') => s.push('"'),
                        Some('\\') => s.push('\\'),
                        Some(d) => return Err(anyhow!("Unknown escape sequence \\{d} in string.")),
                        None => return Err(anyhow!("String ends with an unescaped '\\'")),
                    }
                } else {
                    s.push(c);
                }
            }
        } else if c.is_numeric() {
            tokens.push(lex_int(c, &mut cs)?);
        } else if c == '-' {
            if let Some(next) = cs.peek() {
                if next.is_numeric() {
                    tokens.push(lex_int(c, &mut cs)?);
                } else {
                    tokens.push(Token::Dash);
                }
            }
        } else if c.is_whitespace() {
        } else {
            return Err(anyhow!("Unexpected character: {}", c));
        }
    }

    Ok(tokens)
}

fn lex_int(c: char, cs: &mut Peekable<Characters>) -> Result<Token> {
    let mut i = String::from(c);
    loop {
        match cs.peek() {
            Some(p) if p.is_numeric() => i.push(cs.next().unwrap()),
            Some(p) if *p == '_' => {
                cs.next();
            }
            _ => break Ok(Token::Int(i.parse()?)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Token::*;
    #[test]

    fn bools() -> Result<()> {
        assert_eq!(
            vec![
                LeftParen,
                Test,
                String(r#"a "test" \named\ test"#.to_owned()),
                LeftParen,
                Assert,
                Boolean(true),
                Boolean(false),
                RightParen,
                RightParen
            ],
            lex(r#"(test "a \"test\" \\named\\ test" (assert true false))"#)?
        );
        Ok(())
    }

    #[test]
    fn slash() -> Result<()> {
        assert_eq!(vec![Slash], lex("/")?);
        Ok(())
    }

    #[test]
    fn integers() -> Result<()> {
        assert_eq!(
            vec![Int(1), Int(2134234), Int(-12534546), Int(1_000_000)],
            lex("1 2134234 -12534546 1_000_000")?
        );
        Ok(())
    }

    #[test]
    fn asserteq() -> Result<()> {
        assert_eq!(vec![AssertEq], lex("asserteq")?);
        Ok(())
    }

    #[test]
    fn arithmetic() -> Result<()> {
        assert_eq!(vec![Plus, Dash, Star, Slash], lex("+ - * /")?);
        Ok(())
    }

    #[test]
    fn r#let() -> Result<()> {
        assert_eq!(vec![Let], lex("let")?);
        Ok(())
    }
}

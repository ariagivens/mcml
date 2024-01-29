use std::fmt::Display;

use anyhow::{Result, anyhow};
use crate::utility::escape;

#[derive(PartialEq, Eq, Debug)]
pub enum Token {
    LeftParen,
    RightParen,
    Slash,
    Ident(String),
    Test,
    Assert,
    Boolean(bool),
    String(String),
}

impl Token {
    fn ident(s: String) -> Token {
        if &s == "test" {
            Token::Test
        } else if &s == "assert" {
            Token::Assert
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
            Token::Slash => write!(f, "/"),
            Token::Ident(i) => write!(f, "{}", i),
            Token::Test => write!(f, "test"),
            Token::Assert => write!(f, "assert"),
            Token::Boolean(b) => {
                if *b {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
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
        } else if c.is_whitespace() {
        } else {
            return Err(anyhow!("Unexpected character: {}", c));
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod test {
    use super::*;
    use Token::*;
    #[test]

fn lex_test() -> Result<()> {
    
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
fn test_lex_slash() -> Result<()> {
    use Token::*;
    assert_eq!(
        vec![ Slash ],
        lex("/")?
    );
    Ok(())
}

}


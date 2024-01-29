use anyhow::{Result, anyhow};

use crate::datapack::Function;
use crate::parse::{Definition, Expr, Statement};
use crate::utility::escape;

pub fn codegen(defs: Vec<Definition>) -> Result<Vec<Function>> {
    let mut functions = Vec::new();

    let preamble = format!("tellraw @s \"TAP version 14\"\ntellraw @s \"1..{}\"\n", defs.len());

    let mut tests = String::new();
    for (i, def) in defs.iter().enumerate() {
        let f = codegen_def(def, i)?;
        functions.push(f);
        tests.push_str(&format!("function mctest:test{}\n", i));
    }

    functions.push(Function {
        namespace: "mctest".to_owned(),
        name: "run".to_owned(),
        content: format!("{}\n{}\ntellraw @s \"<EOF>\"", preamble, tests),
    });

    Ok(functions)
}

fn codegen_def(def: &Definition, i: usize) -> Result<Function> {
    let Definition::Test { name, stmt } = def;
    
    let cmd = match stmt {
        Statement::Assert {
            expr: Expr::LitBool(b),
        } => format!(
            r#"tellraw @s "{} - {}""#,
            if *b { "ok" } else { "not ok" },
            escape(&name)
        ),
        Statement::Assert {
            expr: Expr::LitInt(_),
        } => return Err(anyhow!("Expected (assert <bool>), but saw (assert <int>) instead")),
        Statement::AssertEq { left: Expr::LitBool(a), right: Expr::LitBool(b) } =>
        format!(
            r#"tellraw @s "{} - {}""#,
            if a == b { "ok" } else { "not ok" },
            escape(&name)
        ),
        Statement::AssertEq { left: Expr::LitInt(a), right: Expr::LitInt(b) } =>
        format!(
            r#"tellraw @s "{} - {}""#,
            if a == b { "ok" } else { "not ok" },
            escape(&name)
        ),
        Statement::AssertEq { .. } => return Err(anyhow!("Tried to compare values of different types")),
        Statement::Command { text } => text.to_owned(),
    };

    Ok(Function {
        namespace: "mctest".to_owned(),
        name: format!("test{}", i),
        content: cmd,
    })
}
use crate::datapack::Function;
use crate::parse::{ASTNode, Expr, Statement};
use crate::utility::escape;

pub fn codegen(node: ASTNode) -> Vec<Function> {
    let ASTNode::Test { name, stmt } = node;

    let cmd = match stmt {
        Statement::Assert {
            expr: Expr::LitBool(b),
        } => format!(
            r#"tellraw @s "{} - {}""#,
            if b { "ok" } else { "not ok" },
            escape(&name)
        ),
        Statement::Command { text } => text,
    };

    vec![
        Function {
            namespace: "mctest".to_owned(),
            name: "list".to_owned(),
            content: r#"tellraw @s "/function mctest:test1""#.to_owned(),
        },
        Function {
            namespace: "mctest".to_owned(),
            name: "test1".to_owned(),
            content: cmd,
        },
        Function {
            namespace: "mctest".to_owned(),
            name: "plan".to_owned(),
            content: r#"tellraw @s "1..1""#.to_owned(),
        },
    ]
}

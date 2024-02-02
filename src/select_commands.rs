use anyhow::{anyhow, Result};

use crate::assign_homes::{self, Atom, Binary, Block, Expr, Location, Statement};
use crate::datapack::Function;
use crate::utility::escape;

pub fn select_commands(program: assign_homes::Program) -> Result<Vec<Function>> {
    let mut functions = Vec::new();

    let mut preamble = "tellraw @s \"TAP version 14\"\n".to_owned();
    preamble.push_str(&format!("tellraw @s \"1..{}\"\n", program.tests.len()));
    preamble.push_str(&format!("scoreboard objectives add stack dummy"));

    let mut tests = String::new();
    for (i, t) in program.tests.iter().enumerate() {
        let block = program.blocks[t.block].clone();
        let content = select_commands_block(block, &escape(&t.name));
        functions.push(Function {
            namespace: "mctest".to_owned(),
            name: format!("test{i}"),
            content,
        });
        tests.push_str(&format!("function mctest:test{}\n", i));
    }

    functions.push(Function {
        namespace: "mctest".to_owned(),
        name: "run".to_owned(),
        content: format!("{}\n{}\ntellraw @s \"<EOF>\"", preamble, tests),
    });

    Ok(functions)
}

fn select_commands_block(block: Block, test_name: &str) -> String {
    let stmts = block.stmts;
    let mut text = String::new();
    for stmt in stmts {
        text.push_str(&select_commands_stmt(stmt, test_name));
    }
    text
}

fn select_commands_stmt(stmt: Statement, test_name: &str) -> String {
    match stmt {
        Statement::Assert {
            atom: Atom::LitBool(b),
        } => format!(
            "tellraw @s \"{} - {}\"\n",
            if b { "ok" } else { "not ok" },
            escape(test_name)
        ),
        Statement::Assert {
            atom: Atom::Location(Location::Stack { offset }),
        } => {
            let ok = format!(
                "execute if score {offset} stack matches 1.. run tellraw @s \"ok - {test_name}\""
            );
            let not_ok = format!("execute unless score {offset} stack matches 1.. run tellraw @s \"not ok - {test_name}\"");
            format!("{ok}\n{not_ok}\n")
        }
        Statement::Assert {
            atom: Atom::LitInt(i),
        } => panic!("Type error! Expected <bool> saw <int>"),
        Statement::AssertEq {
            left: Atom::LitBool(left),
            right: Atom::LitBool(right),
        } => format!(
            "tellraw @s \"{} - {}\"\n",
            if left == right { "ok" } else { "not ok" },
            escape(test_name)
        ),
        Statement::AssertEq {
            left: Atom::LitInt(left),
            right: Atom::LitInt(right),
        } => format!(
            "tellraw @s \"{} - {}\"\n",
            if left == right { "ok" } else { "not ok" },
            escape(test_name)
        ),
        Statement::AssertEq {
            left: Atom::Location(Location::Stack { offset }),
            right: Atom::LitInt(i),
        } => {
            let ok = format!(
                "execute if score {offset} stack matches {i} run tellraw @s \"ok - {test_name}\""
            );
            let not_ok = format!("execute unless score {offset} stack matches {i} run tellraw @s \"not ok - {test_name}\"");
            format!("{ok}\n{not_ok}\n")
        }
        Statement::AssertEq {
            left: Atom::LitInt(i),
            right: Atom::Location(Location::Stack { offset }),
        } => {
            let ok = format!(
                "execute if score {offset} stack matches {i} run tellraw @s \"ok - {test_name}\""
            );
            let not_ok = format!("execute unless score {offset} stack matches {i} run tellraw @s \"not ok - {test_name}\"");
            format!("{ok}\n{not_ok}\n")
        }
        Statement::AssertEq {
            left:
                Atom::Location(Location::Stack {
                    offset: left_offset,
                }),
            right:
                Atom::Location(Location::Stack {
                    offset: right_offset,
                }),
        } => {
            let ok = format!("execute if score {left_offset} stack = {right_offset} stack run tellraw @s \"ok - {test_name}\"");
            let not_ok = format!("execute unless score {left_offset} stack = {right_offset} stack run tellraw @s \"ok - {test_name}\"");
            format!("{ok}\n{not_ok}\n")
        }
        Statement::AssertEq { .. } => panic!("Type error! Tried to compare <bool> and <int>"),
        Statement::Command { text } => format!("{text}\n"),
        Statement::Assign {
            location: Location::Stack { offset },
            expr,
        } => {
            let scratch = scratch_expr(expr);
            let mv = format!("scoreboard players operation {offset} stack = scratch stack");
            format!("{scratch}\n{mv}\n")
        }
    }
}

fn scratch_expr(expr: Expr) -> String {
    match expr {
        Expr::Atom(a) => scratch_atom(a),
        Expr::Plus(binary) => select_commands_binary('+', binary),
        Expr::Minus(binary) => select_commands_binary('-', binary),
        Expr::Times(binary) => select_commands_binary('*', binary),
        Expr::Divide(binary) => select_commands_binary('/', binary),
    }
}

fn select_commands_binary(op: char, Binary { left, right }: Binary) -> String {
    let scratch = scratch_atom(left);
    let action = match right {
        Atom::Location(Location::Stack { offset }) => format!("scoreboard players operation scratch stack {op}= {offset} stack"),
        Atom::LitInt(i) => format!("scoreboard players set scratch2 stack {i}\nscoreboard players operation scratch stack {op}= scratch2 stack"),
        Atom::LitBool(b) => panic!("Tried to do arithmetic on bools???")
    };
    format!("{scratch}\n{action}")
}

fn scratch_atom(atom: Atom) -> String {
    match atom {
        Atom::Location(Location::Stack { offset }) => format!(
            "scoreboard players operation scratch stack = {} stack\n",
            offset
        ),
        Atom::LitInt(i) => format!("scoreboard players set scratch stack {i}\n"),
        Atom::LitBool(b) => format!(
            "scoreboard players set scratch stack {}\n",
            if b { 1 } else { 0 }
        ),
    }
}

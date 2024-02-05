use anyhow::Result;

use crate::assign_homes::{self, Atom, Binary, Block, Expr, Location, Statement};
use crate::datapack::Function;
use crate::runtime::{setup_runtime, Runtime};
use crate::utility::escape;

pub fn select_commands(program: assign_homes::Program) -> Result<Vec<Function>> {
    let Runtime {
        init: mut preamble,
        mut functions,
    } = setup_runtime();

    preamble.push_str(&format!("tellraw @s \"TAP version 14\"\n"));
    preamble.push_str(&format!("tellraw @s \"1..{}\"\n", program.tests.len()));
    preamble.push_str(&format!("scoreboard players set ptr stack 10\n"));

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
            let setup =
                format!("scoreboard players set offset stack {offset}\nfunction mctest:pop\n");
            let ok = format!(
                "execute if score item stack matches 1.. run tellraw @s \"ok - {test_name}\"\n"
            );
            let not_ok = format!("execute unless score item stack matches 1.. run tellraw @s \"not ok - {test_name}\"\n");
            format!("{setup}{ok}{not_ok}")
        }
        Statement::Assert {
            atom: Atom::LitInt(_),
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
            let setup =
                format!("scoreboard players set offset stack {offset}\nfunction mctest:pop\n");
            let ok = format!(
                "execute if score item stack matches {i} run tellraw @s \"ok - {test_name}\"\n"
            );
            let not_ok = format!("execute unless score item stack matches {i} run tellraw @s \"not ok - {test_name}\"\n");
            format!("{setup}{ok}{not_ok}")
        }
        Statement::AssertEq {
            left: Atom::LitInt(i),
            right: Atom::Location(Location::Stack { offset }),
        } => {
            let setup =
                format!("scoreboard players set offset stack {offset}\nfunction mctest:pop\n");
            let ok = format!(
                "execute if score item stack matches {i} run tellraw @s \"ok - {test_name}\"\n"
            );
            let not_ok = format!("execute unless score item stack matches {i} run tellraw @s \"not ok - {test_name}\"\n");
            format!("{setup}{ok}{not_ok}")
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
            let setup_left = format!("scoreboard players set offset stack {left_offset}\nfunction mctest:pop\nscoreboard players operation scratch registry = item stack\n");
            let setup_right = format!(
                "scoreboard players set offset stack {right_offset}\nfunction mctest:pop\n"
            );
            let ok = format!("execute if score scratch registry = item stack run tellraw @s \"ok - {test_name}\"\n");
            let not_ok = format!("execute unless score scratch registry = item stack run tellraw @s \"not ok - {test_name}\"\n");
            format!("{setup_left}{setup_right}{ok}{not_ok}")
        }
        Statement::AssertEq { .. } => panic!("Type error! Tried to compare <bool> and <int>"),
        Statement::Command { text } => format!("{text}\n"),
        Statement::Assign {
            location: Location::Stack { offset },
            expr,
        } => {
            let scratch = scratch_expr(expr);
            let mv = format!("scoreboard players set offset stack {offset}\nscoreboard players operation item stack = scratch registry\nfunction mctest:push\n");
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
        Atom::Location(Location::Stack { offset }) => format!("scoreboard players set offset stack {offset}\nfunction mctest:pop\nscoreboard players operation scratch registry {op}= item stack"),
        Atom::LitInt(i) => format!("scoreboard players set scratch2 registry {i}\nscoreboard players operation scratch registry {op}= scratch2 registry"),
        Atom::LitBool(_) => panic!("Tried to do arithmetic on bools???")
    };
    format!("{scratch}\n{action}")
}

fn scratch_atom(atom: Atom) -> String {
    match atom {
        Atom::Location(Location::Stack { offset }) => format!(
            "scoreboard players set offset stack {offset}\nfunction mctest:pop\nscoreboard players operation scratch registry = item stack\n"
        ),
        Atom::LitInt(i) => format!("scoreboard players set scratch registry {i}\n"),
        Atom::LitBool(b) => format!(
            "scoreboard players set scratch registry {}\n",
            if b { 1 } else { 0 }
        ),
    }
}

use anyhow::Result;

use crate::datapack::Function;
use crate::reify_locations::{self as prev, Run};
use crate::runtime::{setup_runtime, Runtime};
use crate::utility::escape;

pub fn emit_text(program: prev::Program) -> Vec<Function> {
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
        let content = emit_text_block(block);
        functions.push(Function {
            namespace: "mctest".to_owned(),
            name: format!("test{i}"),
            content,
        });
        tests.push_str(&format!("function mctest:test{}\n", i));
    }

    for idx in program.blocks.node_indices().filter(|idx| program.tests.iter().map(|t| t.block).find(|i| i == idx).is_none()) {
        let block = program.blocks[idx].clone();
        let content = emit_text_block(block);
        functions.push(Function {
            namespace: "mctest".to_owned(),
            name: format!("block{}", idx.index()),
            content
        });
    }

    functions.push(Function {
        namespace: "mctest".to_owned(),
        name: "run".to_owned(),
        content: format!("{}\n{}\ntellraw @s \"<EOF>\"", preamble, tests),
    });

    functions
}

fn emit_text_block(block: prev::Block) -> String {
    let mut text = String::new();
    for instr in block.instrs {
        text.push_str(&emit_text_instr(instr));
    }
    text
}

fn emit_text_instr(instr: prev::Instruction) -> String {
    match instr {
        prev::Instruction::Set { location, value } => {
            format!("scoreboard players set {location} {value}\n")
        }
        prev::Instruction::Operation {
            op,
            source,
            destination,
        } => format!("scoreboard players operation {destination} {op} {source}\n"),
        prev::Instruction::Push { offset } => {
            format!("scoreboard players set offset stack {offset}\nfunction mctest:push\n")
        }
        prev::Instruction::Pop { offset } => {
            format!("scoreboard players set offset stack {offset}\nfunction mctest:pop\n")
        }
        prev::Instruction::Tellraw { text } => format!("tellraw @s \"{}\"\n", escape(&text)),
        prev::Instruction::Command { text } => format!("{text}\n"),
        prev::Instruction::ExecuteIfScoreMatches {
            location,
            value,
            run,
        } => format!(
            "execute if score {location} matches {value} run {}\n",
            emit_text_run(run),
        ),
        prev::Instruction::ExecuteUnlessScoreMatches {
            location,
            value,
            run,
        } => format!(
            "execute unless score {location} matches {value} run {}\n",
            emit_text_run(run)
        ),
        prev::Instruction::ExecuteIfScoreEquals { a, b, run } => todo!(),
        prev::Instruction::ExecuteUnlessScoreEquals { a, b, run } => todo!(),
        prev::Instruction::Function { block } => format!("function mctest:block{}\n", block.index()),
    }
}

fn emit_text_run(run: prev::Run) -> String {
    match run {
        Run::Function { block } => format!("function mctest:block{}", block.index()),
        Run::Set { location, value } => format!("scoreboard players set {location} {value}"),
    }
}
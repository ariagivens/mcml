use anyhow::Result;
use clap::Parser;
use mcml::compile;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    input: PathBuf,
    output: PathBuf,
}

fn main() -> Result<()> {
    let Args { input, output } = Args::parse();
    let source = fs::read_to_string(input)?;
    let datapack = compile(&source)?;
    fs::write(output, datapack.bytes()?)?;

    Ok(())
}

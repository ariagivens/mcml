use anyhow::{anyhow, Context, Result};
use std::process;
use std::{ffi::OsStr, fs};
use tap_parser::{TapParser, TapStatement};
use tempdir::TempDir;

fn mctest(path: &impl AsRef<OsStr>) -> Result<()> {
    let output = process::Command::new("../mctest/target/release/mctest")
        .arg(path)
        .output()
        .context("Failed to launch mctest.")?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failure running test: {}",
            String::from_utf8(output.stderr)?
        ));
    }

    let document = String::from_utf8(output.stdout)?;
    let tap = TapParser::new()
        .parse(&document)
        .with_context(|| format!("Failed to parse mctest output:\n{}<EOF>", &document))?;

    for stmt in tap {
        match stmt {
            TapStatement::TestPoint(t) => assert!(t.result),
            TapStatement::Subtest(t) => assert!(t.ending.result),
            _ => {}
        }
    }

    Ok(())
}

fn run_test(program: &str) -> Result<()> {
    let tempdir = TempDir::new("mcml_test")?;
    let pack_path = tempdir.path().join("pack.zip");
    let contents = mcml::compile(program)?.bytes();
    fs::write(&pack_path, &contents?)?;
    mctest(&pack_path)?;

    Ok(())
}

#[test]
fn literals() -> Result<()> {
    run_test(include_str!("literals.mcml"))
}

#[test]
#[should_panic]
fn fail() {
    run_test(include_str!("fail.mcml")).unwrap();
}

#[test]
fn hello_world() -> Result<()> {
    run_test(include_str!("commands.mcml"))
}

#[test]
fn arithmetic() -> Result<()> {
    run_test(include_str!("arithmetic.mcml"))
}

#[test]
fn variables() -> Result<()> {
    run_test(include_str!("variables.mcml"))
}

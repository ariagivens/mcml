mod lex;
mod utility;
mod parse;
mod codegen;
mod datapack;

use lex::lex;
use parse::parse;
use codegen::codegen;
pub use datapack::Datapack;
use anyhow::Result;

pub fn compile(source: &str) -> Result<Datapack> {
    let functions = codegen(parse(lex(source)?)?);

    Ok(Datapack {
        description: "Datapack generated by MCML".to_owned(),
        pack_format: 18,
        functions,
    })
}

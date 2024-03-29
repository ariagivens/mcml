mod assign_homes;
mod datapack;
mod desugar_asserts;
mod emit_text;
mod lex;
mod linearize;
mod parse;
mod reify_locations;
mod runtime;
mod select_instructions;
mod uniquify;
mod utility;
mod var;
mod insert_jmps;

use anyhow::Result;
use assign_homes::assign_homes;
pub use datapack::Datapack;
use desugar_asserts::desugar_asserts;
use emit_text::emit_text;
use lex::lex;
use linearize::linearize;
use parse::parse;
use reify_locations::reify_location;
use select_instructions::select_instructions;
use tap::pipe::Pipe;
use uniquify::uniquify;
use insert_jmps::insert_jmps;

pub fn compile(source: &str) -> Result<Datapack> {
    let functions = source
        .pipe(lex)?
        .pipe(parse)?
        .pipe(uniquify)
        .pipe(desugar_asserts)
        .pipe(linearize)
        .pipe(select_instructions)
        .pipe(assign_homes)
        .pipe(insert_jmps)
        .pipe(reify_location)
        .pipe(emit_text);

    Ok(Datapack {
        description: "Datapack generated by MCML".to_owned(),
        pack_format: 18,
        functions,
    })
}

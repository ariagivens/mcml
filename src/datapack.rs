use anyhow::Result;
use json::object;
use std::io::{Cursor, Write};
use zip::ZipWriter;

pub struct Datapack {
    pub description: String,
    pub pack_format: usize,
    pub functions: Vec<Function>,
}

pub struct Function {
    pub namespace: String,
    pub name: String,
    pub content: String,
}

impl Datapack {
    pub fn bytes(&self) -> Result<Vec<u8>> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file("pack.mcmeta", options)?;
        let metadata = object! {
            "pack": object!{
                "description": self.description.to_owned(),
                "pack_format": self.pack_format.to_owned()
            }
        };
        write!(zip, "{}", json::stringify(metadata))?;

        for function in &self.functions {
            zip.start_file(
                format!(
                    "data/{}/functions/{}.mcfunction",
                    function.namespace, function.name
                ),
                options,
            )?;
            write!(zip, "{}", function.content)?;
        }

        Ok(zip.finish()?.into_inner())
    }
}

pub mod config;
pub mod daemon;
pub mod filesystem;
pub mod rule;
pub mod subvolume;
pub mod ui;

pub fn write_as_json<S: serde::Serialize>(path: &std::path::Path, data: &S) -> std::io::Result<()> {
    use std::{
        fs::File,
        io::{BufWriter, Write},
    };
    let mut f = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(&mut f, data)?;
    f.write(b"\n")?;
    f.flush()?;
    Ok(())
}

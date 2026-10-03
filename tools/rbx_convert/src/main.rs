//! Offline source conversion only; runtime physics stays dependency-free.
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: avatar-rbx-convert input.rbxm output.rbxmx".into());
    }
    if std::fs::metadata(&args[1])?.len() > 32 * 1024 * 1024 {
        return Err("source too large".into());
    }
    let dom = rbx_binary::from_reader(BufReader::new(File::open(&args[1])?))?;
    rbx_xml::to_writer(
        BufWriter::new(File::create(&args[2])?),
        &dom,
        dom.root().children(),
        rbx_xml::EncodeOptions::default(),
    )?;
    Ok(())
}

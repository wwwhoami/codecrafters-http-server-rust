use anyhow::Result;
use flate2::{write::GzEncoder, Compression};
use std::io::Write;

pub fn gzip_str(string: &str) -> Result<Vec<u8>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(string.as_bytes())?;
    e.finish().map_err(|e| e.into())
}

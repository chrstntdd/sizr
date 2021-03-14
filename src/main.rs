use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Write;

fn main() -> io::Result<()> {
    let mut input_file = File::open("sfs.js")?;
    let meta = fs::metadata("sfs.js")?;

    let mut buf = Vec::new();
    input_file.read_to_end(&mut buf)?;

    let br_comp_res = br_compress(&buf);
    let gz_comp_res = gz_compress(&buf.clone())?;

    println!(
        "og: {:?} \ngz: {:?} \nbr: {:?}",
        meta.len(),
        gz_comp_res.len(),
        br_comp_res.len()
    );

    Ok(())
}

fn br_compress(input: &[u8]) -> Vec<u8> {
    let mut writer = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
    writer.write_all(input).unwrap();
    writer.into_inner()
}

fn gz_compress(input: &[u8]) -> io::Result<Vec<u8>> {
    let mut gz_enc = ZlibEncoder::new(Vec::new(), Compression::best());
    gz_enc.write_all(input)?;
    gz_enc.finish()
}

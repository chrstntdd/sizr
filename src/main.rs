use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::clap::arg_enum;
use structopt::StructOpt;

mod byte_formatter {
    use std::cmp;

    pub fn pretty(num: f64) -> String {
        let negative = if num.is_sign_positive() { "" } else { "-" };
        let num = num.abs();
        let units = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
        if num < 1_f64 {
            return format!("{}{} {}", negative, num, "B");
        }
        let delimiter = 1000_f64;
        let exponent = cmp::min(
            (num.ln() / delimiter.ln()).floor() as i32,
            (units.len() - 1) as i32,
        );
        let pretty_bytes = format!("{:.2}", num / delimiter.powi(exponent))
            .parse::<f64>()
            .unwrap()
            * 1_f64;
        let unit = units[exponent as usize];
        format!("{}{} {}", negative, pretty_bytes, unit)
    }
}

mod compress {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    use std::io;
    use std::io::prelude::*;
    pub fn br_compress(input: &[u8]) -> Vec<u8> {
        let mut writer = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
        writer.write_all(input).unwrap();
        writer.into_inner()
    }
    pub fn gz_compress(input: &[u8]) -> io::Result<Vec<u8>> {
        let mut gz_enc = ZlibEncoder::new(Vec::new(), Compression::best());
        gz_enc.write_all(input)?;
        gz_enc.finish()
    }
}

arg_enum! {
    #[derive(Debug)]
    enum CompressionKind {
        Gz,
        Br,
        All
    }
}

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(possible_values = &CompressionKind::variants(), case_insensitive = true)]
    kind: CompressionKind,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    let opt = Cli::from_args();

    for file in opt.files {
        let contents = read_input_mod(&file)?;
        let sizes = pretty_sizes(get_length(contents)?);

        println!("sizes: {:?}", sizes);
    }

    Ok(())
}

fn pretty_sizes(x: (usize, usize, usize)) -> (String, String, String) {
    (
        byte_formatter::pretty(x.0 as f64),
        byte_formatter::pretty(x.1 as f64),
        byte_formatter::pretty(x.2 as f64),
    )
}

fn get_length(x: Vec<u8>) -> io::Result<(usize, usize, usize)> {
    let raw_size = x.len();
    let br_comp_res = compress::br_compress(&x);
    let gz_comp_res = compress::gz_compress(&x.clone())?;

    Ok((raw_size, gz_comp_res.len(), br_comp_res.len()))
}

fn read_input_mod(p: &PathBuf) -> io::Result<Vec<u8>> {
    let mut input_file = File::open(p)?;
    let mut buf = Vec::new();
    input_file.read_to_end(&mut buf)?;

    Ok(buf)
}

use ansi_term::Style;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::clap::arg_enum;
use structopt::StructOpt;

mod byte_fmt {
    use std::cmp;

    /// Forked from https://github.com/banyan/rust-pretty-bytes/blob/b164c61b87b633cc175e02b6fddc7d28fcf6b9c7/src/converter.rs
    pub fn pretty(num: f64) -> String {
        let negative = if num.is_sign_positive() { "" } else { "-" };
        let num = num.abs();
        let units = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
        if num < 1_f64 {
            return format!("{}{} {}", negative, num, "B");
        }
        let delimiter = 1024_f64;
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

    pub fn brotli(input: &[u8]) -> io::Result<Vec<u8>> {
        let mut writer = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
        writer.write_all(input)?;
        Result::Ok(writer.into_inner())
    }
    pub fn gzip(input: &[u8]) -> io::Result<Vec<u8>> {
        let mut gz_enc = ZlibEncoder::new(Vec::new(), Compression::best());
        gz_enc.write_all(input)?;
        gz_enc.finish()
    }
}

arg_enum! {
    #[derive(Copy,Clone,Debug)]
    enum CompressionKind {
        Gz,
        Br,
        Raw,
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
    let args = Cli::from_args();

    for file in args.files {
        println!("{}", Style::new().bold().paint(format!("{:?}", &file)));
        let contents = read_input_mod(&file)?;
        match get_sizes(contents, args.kind)? {
            CompressionResult::All { raw, br, gz } => {
                println!("  raw: {}", byte_fmt::pretty(raw as f64));
                println!("  br : {}", byte_fmt::pretty(br as f64));
                println!("  gz : {}", byte_fmt::pretty(gz as f64));
            }
            CompressionResult::Br(b) | CompressionResult::Gz(b) | CompressionResult::Raw(b) => {
                println!("  {:?}: {}", args.kind, byte_fmt::pretty(b as f64));
            }
        };
    }

    Ok(())
}

enum CompressionResult {
    Gz(usize),
    Br(usize),
    Raw(usize),
    All { gz: usize, br: usize, raw: usize },
}

fn get_sizes(x: Vec<u8>, k: CompressionKind) -> io::Result<CompressionResult> {
    match k {
        CompressionKind::Gz => {
            let gz_b = compress::gzip(&x)?.len();

            Ok(CompressionResult::Gz(gz_b))
        }
        CompressionKind::Br => {
            let br_b = compress::brotli(&x)?.len();

            Ok(CompressionResult::Br(br_b))
        }
        CompressionKind::Raw => {
            let raw_b = x.len();

            Ok(CompressionResult::Raw(raw_b))
        }
        CompressionKind::All => {
            let br = compress::brotli(&x)?.len();
            let gz = compress::gzip(&x.clone())?.len();
            let raw = x.len();
            Ok(CompressionResult::All { br, gz, raw })
        }
    }
}

fn read_input_mod(p: &PathBuf) -> io::Result<Vec<u8>> {
    let mut input_file = File::open(p)?;
    let mut buf = Vec::new();
    input_file.read_to_end(&mut buf)?;

    Ok(buf)
}

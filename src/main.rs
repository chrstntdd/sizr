use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::clap::arg_enum;
use structopt::StructOpt;
#[macro_use]
extern crate prettytable;
use prettytable::{Cell, Row, Table};

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
    }
}

#[derive(StructOpt, Debug)]
struct Cli {
    /// If omitted, all 3 sizes are reported
    #[structopt(possible_values = &CompressionKind::variants(), case_insensitive = true, value_delimiter = ",", short)]
    kind: Option<Vec<CompressionKind>>,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = Cli::from_args();
    let mut table = Table::new();
    let mut has_col_labels = false;

    for file in args.files {
        let contents = read_file(&file)?;
        let file_name = &file.to_str().expect("Unable to read the file path");

        match &args.kind {
            Some(compression_kinds) => {
                // TODO: Prevent duplicate formats
                // TODO: allow more ergonomic single encoding option.
                // Right now single encoding still requires the ending ","
                // Ex: `sizr br, ./path/to/file.js`
                //

                // Print appropriate column labels
                if has_col_labels == false {
                    let mut col_labels = vec![Cell::new("name")];
                    for el in compression_kinds {
                        col_labels.append(&mut vec![Cell::new(&format!("{}", el))]);
                    }
                    table.add_row(Row::new(col_labels));
                    has_col_labels = true
                };

                //
                let mut size_col = vec![Cell::new(file_name)];
                for el in compression_kinds {
                    match get_sizes(contents.clone(), *el)? {
                        CompressionResult::Br(b)
                        | CompressionResult::Gz(b)
                        | CompressionResult::Raw(b) => {
                            size_col.append(&mut vec![Cell::new(&byte_fmt::pretty(b as f64))]);
                        }
                    };
                }

                table.add_row(Row::new(size_col));
            }
            None => {
                if has_col_labels == false {
                    table.add_row(row!["name", "raw", "gzip", "brotli"]);
                    has_col_labels = true
                };
                let br = compress::brotli(&contents)?.len();
                let gz = compress::gzip(&contents.clone())?.len();
                let raw = contents.len();
                table.add_row(Row::new(vec![
                    Cell::new(file_name),
                    Cell::new(&byte_fmt::pretty(raw as f64)),
                    Cell::new(&byte_fmt::pretty(br as f64)),
                    Cell::new(&byte_fmt::pretty(gz as f64)),
                ]));
            }
        };
    }
    table.printstd();

    Ok(())
}

#[derive(Copy, Clone, Debug)]
enum CompressionResult {
    Gz(usize),
    Br(usize),
    Raw(usize),
}

impl fmt::Display for CompressionResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CompressionResult::Br(_) => write!(f, "br"),
            CompressionResult::Gz(_) => write!(f, "gz"),
            CompressionResult::Raw(_) => write!(f, "raw"),
        }
    }
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
    }
}

fn read_file(p: &PathBuf) -> io::Result<Vec<u8>> {
    let mut input_file = File::open(p)?;
    let mut buf = Vec::new();
    input_file.read_to_end(&mut buf)?;

    Ok(buf)
}

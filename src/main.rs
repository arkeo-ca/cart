use cart;

use std::process;
use argparse::{ArgumentParser, StoreTrue, Store, StoreOption, Print};

fn main() {
    let config = Config::new().unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    //println!("{:?}", config);

    if config.file.len() == 0 {
        println!("No file specified. Please use 'cart -h' to show help message.");
        process::exit(1);
    }

    let o_path = match config.outfile {
        Some(f) => f,
        None => format!("{}.cart", config.file),
    };


    cart::pack_file(&config.file, &o_path, None, None, None).unwrap_or_else(|err| {
        println!("{}", err);
    });
}

#[derive(Debug)]
struct Config {
    file: String,
    delete: bool,
    force: bool,
    ignore: bool,
    meta: bool,
    showmeta: bool,
    jsonmeta: Option<String>,
    key: Option<String>,
    name: Option<String>,
    outfile: Option<String>
}

impl Config {
    fn new() -> Result<Config, &'static str> {
        let mut file: String = String::from("");

        let mut delete = false;
        let mut force = false;
        let mut ignore = false;
        let mut meta = false;
        let mut showmeta = false;

        let mut jsonmeta: Option<String> = None;
        let mut key: Option<String> = None;
        let mut name: Option<String> = None;
        let mut outfile: Option<String> = None;


        {
            let mut ap = ArgumentParser::new();

            ap.refer(&mut file).add_argument("file", Store, "");
            ap.add_option(&["-v", "--version"], Print(format!("CaRT v{} (Rust)", env!("CARGO_PKG_VERSION").to_string())), "Show program's version number and exit");
            ap.refer(&mut delete).add_option(&["-d", "--delete"], StoreTrue, "Delete original after operation succeeded");
            ap.refer(&mut force).add_option(&["-f", "--force"], StoreTrue, "Replace output file if it already exists");
            ap.refer(&mut ignore).add_option(&["-i", "--ignore"], StoreTrue, "Ignore RC4 key from conf file");
            ap.refer(&mut jsonmeta).add_option(&["-j", "--jsonmeta"], StoreOption, "Provide header metadata as JSON blob");
            ap.refer(&mut key).add_option(&["-k", "--key"], StoreOption, "Use private RC4 key (base64 encoded). Same key must be provided to unCaRT.");
            ap.refer(&mut meta).add_option(&["-m", "--meta"], StoreTrue, "Keep metadata around when extracting CaRTs");
            ap.refer(&mut name).add_option(&["-n", "--name"], StoreOption, "Use this value as metadata filename");
            ap.refer(&mut outfile).add_option(&["-o", "--outfile"], StoreOption, "Set output file");
            ap.refer(&mut showmeta).add_option(&["-s", "--showmeta"], StoreTrue, "Only show the file metadata");

            ap.parse_args_or_exit();
        }

        Ok(Config {file, delete, force, ignore, meta, showmeta, jsonmeta, key, name, outfile})
    }
}

use std::path::Path;
use configparser::ini::Ini;
use argparse::{ArgumentParser, StoreTrue, List, StoreOption, Print};
use json::JsonValue;

#[derive(Debug)]
pub struct Config {
    pub file: Vec<String>,
    pub delete: bool,
    pub force: bool,
    pub ignore: bool,
    pub meta: bool,
    pub showmeta: bool,
    pub jsonmeta: Option<String>,
    pub key: Option<String>,
    pub name: Option<String>,
    pub outfile: Option<String>,
    pub default_header: JsonValue,
}

impl Config {
    pub fn new() -> Result<Config, Box<dyn std::error::Error>> {
        let mut file = Vec::new();

        let mut delete = false;
        let mut force = false;
        let mut ignore = false;
        let mut meta = false;
        let mut showmeta = false;

        let mut jsonmeta: Option<String> = None;
        let mut key: Option<String> = None;
        let mut name: Option<String> = None;
        let mut outfile: Option<String> = None;

        let mut default_header = JsonValue::new_object();

        {
            let env_home = format!("{}/.cart/cart.cfg", std::env::var("HOME").unwrap());
            let c_path = Path::new(&env_home);

            let mut cp = Ini::new();
            let map = cp.load(c_path);

            if let Ok(m) = map {
                if let Some(v) = cp.getbool("global", "keep_meta").unwrap() {
                    meta = v;
                }
                if let Some(v) = cp.getbool("global", "force").unwrap() {
                    force = v;
                }
                if let Some(v) = cp.getbool("global", "delete").unwrap() {
                    delete = v;
                }
                key = cp.get("global", "rc4_key");

                default_header = json::from(m["default_header"].clone());
            }
        }

        {
            let mut ap = ArgumentParser::new();

            ap.refer(&mut file).add_argument("file", List, "");
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

        if ignore {
            key = None;
        }

        Ok(Config {file, delete, force, ignore, meta, showmeta, jsonmeta, key, name, outfile, default_header})
    }
}


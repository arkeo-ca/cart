use std::path::Path;
use configparser::ini::Ini;
use json::JsonValue;
use clap::Parser;

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
        let mut args = Args::parse();
        
        let mut default_header = JsonValue::new_object();
        {
            #[cfg(target_os = "linux")]
            let env_home = format!("{}/.config/cart/cart.cfg", std::env::var("HOME").unwrap());
            #[cfg(target_os = "windows")]
            let env_home = format!("{}\\Cart\\cart.cfg", std::env::var("APPDATA").unwrap());

            let c_path = Path::new(&env_home);

            let mut config = Ini::new();
            let map = config.load(c_path);

            if let Ok(m) = map {
                if let Some(v) = config.getbool("global", "keep_meta").unwrap() {
                    if !args.meta {
                        args.meta = v;
                    }
                }
                if let Some(v) = config.getbool("global", "force").unwrap() {
                    if !args.force {
                        args.force = v;
                    }
                }
                if let Some(v) = config.getbool("global", "delete").unwrap() {
                    if !args.delete {
                        args.delete = v;
                    }
                }
                if args.key == None {
                    args.key = config.get("global", "rc4_key");
                }
                
                if m.contains_key("default_header") {
                    default_header = json::from(m["default_header"].clone());
                }
            }
        }

        if args.ignore {
            args.key = None;
        }

        Ok(Config {file: args.file, delete: args.delete, force: args.force, ignore: args.ignore, 
            meta: args.meta, showmeta: args.showmeta, jsonmeta: args.jsonmeta, key: args.key, 
            name: args.name, outfile: args.outfile, default_header})
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    file: Vec<String>,

    /// Replace output file if it already exists
    #[clap(short, long)]
    force: bool,

    /// Ignore RC4 key from conf file
    #[clap(short, long)]
    ignore: bool,

    /// Keep metadata around when extracting 
    #[clap(short, long)]
    meta: bool,

    /// Delete original after operation succeeded
    #[clap(short, long)]
    delete: bool,

    /// Only show the file metadata
    #[clap(short, long)]
    showmeta: bool,

    /// Provide header metadata as json blob
    #[clap(short, long)]
    jsonmeta: Option<String>,
    
    /// Use private RC4 key (base64 encoded). Same key must be provided to unCaRT.
    #[clap(short, long)]
    key: Option<String>,
    
    /// Use this value as metadata filename
    #[clap(short, long)]
    name: Option<String>,
    
    /// Set output file
    #[clap(short, long)]
    outfile: Option<String>,
}
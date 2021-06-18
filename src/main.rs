use cart;

use std::process;
use std::fs::{read_to_string, remove_file};
use std::path::Path;
use argparse::{ArgumentParser, StoreTrue, Store, StoreOption, Print};
use base64::decode;
use json::JsonValue;

fn main() {
    // Parse configuration variables
    let config = Config::new().unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    // Ensure that at least one file is provided
    if config.file.len() == 0 {
        println!("No file specified. Please use 'cart -h' to show help message.");
        process::exit(1);
    }

    // Grab provided key from command line and pad as necessary
    let mut arc4key = match config.key {
        Some(k) => Some(decode(k).unwrap_or_else(|_| {
            println!("Could not decode provided RC4 key");
            process::exit(1);
        })),
        None => None,
    };
    if let Some(k) = &mut arc4key {
        let padding_len = 16 - k.len();
        k.extend(vec![0 as u8; padding_len]);
    }

    // Process provided file
    let i_path = Path::new(&config.file);
    if cart::is_cart_file(&i_path).unwrap_or_else(|_| {
        println!("ERR: File '{}' does not exists", &i_path.to_string_lossy());
        process::exit(1);
    }) {
        if config.showmeta {
            let metadata = cart::examine_file(i_path, arc4key);
            match metadata {
                Ok(s) => {
                    println!("{}", s);
                },
                Err(err) => {
                    println!("ERR: Problem parsing metadata ({})", err);
                }
            }
            process::exit(0);
        }

        let o_path = match config.outfile {
            Some(f) => f,
            None => {
                if config.file.ends_with(".cart") {
                    String::from(&config.file[0..config.file.len() - 5])
                } else {
                    String::from(format!("{}.uncart", config.file))
                }
            },
        };
        let o_path = Path::new(&o_path);

        if o_path.is_file() && !config.force {
            println!("ERR: File '{}' already exists", o_path.to_string_lossy());
            process::exit(1);
        }

        cart::unpack_file(i_path, o_path, arc4key).unwrap_or_else(|err| {
            println!("{}", err);
        });

        if config.delete {
            remove_file(&i_path).unwrap_or_else(|_| {
                println!("ERR: Could not delete original file");
                process::exit(1);
            });
        }
    } else {
        // Compile metadata from CLI
        let mut metadata = match config.jsonmeta {
            Some(j) => json::parse(&j).unwrap_or_else(|_| {
                println!("ERR: Invalid JSON metadata");
                process::exit(1);
            }),
            None => JsonValue::new_object(),
        };

        // Compile metadata from cartmeta file
        let m_path = format!("{}.cartmeta", config.file);
        let m_path = Path::new(&m_path);
        if m_path.is_file() {
            let contents = read_to_string(m_path).unwrap();
            for (k, v) in json::parse(&contents).unwrap().entries() {
                metadata.insert(k, v.to_string()).unwrap();
            }
        }

        // Only show metadata if requested
        if config.showmeta {
            println!("{}", metadata.pretty(4));
            process::exit(0);
        }

        // Assign provided filename to metadata if needed
        let name = match config.name {
            Some(n) => n,
            None => i_path.file_name().unwrap().to_str().unwrap().to_string(),
        };
        metadata.insert("name", name).unwrap();

        // Generate and validate output path
        let o_path = match config.outfile {
            Some(f) => f,
            None => format!("{}.cart", config.file),
        };
        let o_path = Path::new(&o_path);

        if o_path.is_file() && !config.force {
            println!("ERR: File '{}' already exists", o_path.to_string_lossy());
            process::exit(1);
        }

        // Pack the file into CaRT format
        cart::pack_file(&i_path, &o_path, Some(metadata.dump()), None, arc4key).unwrap_or_else(|err| {
            println!("{}", err);
        });

        // Remove original file if requested
        if config.delete {
            remove_file(&i_path).unwrap_or_else(|_| {
                println!("ERR: Could not delete original file");
                process::exit(1);
            });
        }
    }
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

use cart;
use base64;

mod config;

use std::process;
use std::fs::{read_to_string, remove_file, write};
use std::path::Path;
use json::JsonValue;

fn main() {
    // Parse configuration variables
    let params = config::Config::new().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    // Validate configuration variables
    if params.file.is_empty() {
        eprintln!("No file specified. Please use 'cart -h' to show help message.");
        process::exit(1);
    }
    if params.file.len() > 1 {
        if let Some(_) = params.name {
            eprintln!("ERR: Cannot set 'filename' option when UN/CaRTing multiple files");
            process::exit(1);
        }
        if let Some(_) = params.outfile {
            eprintln!("ERR: Cannot set 'outfile' option when UN/CaRTing multiple files");
            process::exit(1);
        }
    }

    // Grab provided key from command line and pad as necessary
    let arc4key = params.key.map(|k| {
        let mut key = base64::decode(k).unwrap_or_else(|_| {
            eprintln!("ERR: Could not decode provided RC4 key");
            process::exit(1);
        });

        if key.len() < 16 {
            key.extend(vec![0 as u8; 16 - key.len()]);
        }

        key[0..16].to_vec()
    });

    // Iterate through files
    for input_file in params.file {

        // Validate input file
        let i_path = Path::new(&input_file);
        if !i_path.is_file() {
            eprintln!("ERR: '{}' is not a file", i_path.to_string_lossy());
            process::exit(1);
        }

        // Process provided file
        if cart::is_cart_file(&i_path).unwrap_or_else(|_| {
            eprintln!("ERR: '{}' does not exists", i_path.to_string_lossy());
            process::exit(1);
        }) {
            // Extract metadata from CaRT and print to screen
            if params.showmeta {
                match cart::examine_file(i_path, arc4key.clone()) {
                    Ok(s) => println!("{}", s.pretty(4)),
                    Err(err) => eprintln!("ERR: Problem parsing metadata ({})", err),
                }
                continue;
            }

            // Generate and validate output path
            let o_path = params.outfile.clone().unwrap_or(
                if i_path.ends_with("cart") {
                    i_path.with_extension("").to_string_lossy().to_string()
                } else {
                    format!("{}.uncart", i_path.to_string_lossy())
                }
            );
            let o_path = Path::new(&o_path);
            if o_path.is_file() && !params.force {
                eprintln!("ERR: File '{}' already exists", o_path.to_string_lossy());
                process::exit(1);
            }

            // Unpack CaRTed file
            let metadata = cart::unpack_file(i_path, o_path, arc4key.clone()).unwrap_or_else(|err| {
                eprintln!("ERR: Encountered error during unpacking ({})", err);
                process::exit(1);
            });

            // Write the cartmeta file, if required
            if params.meta {
                let m_path = i_path.with_extension("cartmeta");
                let m_path = Path::new(&m_path);

                write(m_path, metadata.dump()).unwrap_or_else(|_| {
                    eprintln!("ERR: Could not create metadata file");
                    process::exit(1);
                });
            }
        } else {
            // Generate default filename from input path
            let mut metadata = JsonValue::new_object();

            // Compile metadata from cartmeta file (simply carry on if there is no cartmeta file)
            let m_path = format!("{}.cartmeta", i_path.to_string_lossy());
            let m_path = Path::new(&m_path);

            if m_path.is_file() {
                let contents = read_to_string(m_path).unwrap_or_else(|err|{
                    eprintln!("ERR: Could not read cartmeta file ({})", err);
                    process::exit(1);
                });
                let json_contents = json::parse(&contents).unwrap_or_else(|err|{
                    eprintln!("ERR: Could not parse cartmeta file ({})", err);
                    process::exit(1);
                });
                for (k, v) in json_contents.entries() {
                    metadata.insert(k, v.to_string()).unwrap();
                }
            }

            // Compile metadata from CLI
            if let Some(j) = &params.jsonmeta {
                let json_contents = json::parse(&j).unwrap_or_else(|_| {
                    eprintln!("ERR: Invalid JSON metadata");
                    process::exit(1);
                });
                for (k, v) in json_contents.entries() {
                    metadata.insert(k, v.to_string()).unwrap();
                }
            }

            // Compile metadata from config file
            for (k, v) in params.default_header.entries() {
                metadata.insert(k, v.to_string()).unwrap();
            }

            // Assign provided filename to metadata if needed
            if let Some(n) = params.name.clone() {
                metadata.insert("name", n).unwrap();
            }

            // Only show metadata if requested
            if params.showmeta {
                println!("{}", metadata.pretty(4));
                continue;
            }

            // Remove default footer metadata to avoid duplication
            metadata.remove("length");
            metadata.remove("md5");
            metadata.remove("sha1");
            metadata.remove("sha256");

            // Generate and validate output path
            let o_path = params.outfile.clone().unwrap_or(format!("{}.cart", i_path.to_string_lossy()));
            let o_path = Path::new(&o_path);
            if o_path.is_file() && !params.force {
                eprintln!("ERR: File '{}' already exists", o_path.to_string_lossy());
                process::exit(1);
            }

            // Pack the file into CaRT format
            cart::pack_file(&i_path, &o_path, Some(metadata), None, arc4key.clone()).unwrap_or_else(|err| {
                eprintln!("ERR: Encountered error during packing ({})", err);
                process::exit(1);
            });
        }

        // Remove original file if requested
        if params.delete {
            remove_file(&i_path).unwrap_or_else(|_| {
                eprintln!("ERR: Could not delete original file");
                process::exit(1);
            });
        }
    }
}

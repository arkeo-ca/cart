use cart;
mod config;

use std::process;
use std::fs::{read_to_string, remove_file, write};
use std::path::Path;
use base64;
use json::JsonValue;

fn main() {
    // Parse configuration variables
    let params = config::Config::new().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    // Ensure that at least one file is provided
    if params.file.len() == 0 {
        eprintln!("No file specified. Please use 'cart -h' to show help message.");
        process::exit(1);
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

    // Validate input file
    let i_path = Path::new(&params.file);
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
            match cart::examine_file(i_path, arc4key) {
                Ok(s) => println!("{}", s),
                Err(err) => eprintln!("ERR: Problem parsing metadata ({})", err),
            }
            process::exit(0);
        }

        // Generate and validate output path
        let o_path = params.outfile.unwrap_or(
            if params.file.ends_with(".cart") {
                String::from(&params.file[0..params.file.len() - 5])
            } else {
                format!("{}.uncart", params.file)
            }
        );
        let o_path = Path::new(&o_path);
        if o_path.is_file() && !params.force {
            eprintln!("ERR: File '{}' already exists", o_path.to_string_lossy());
            process::exit(1);
        }

        // Unpack CaRTed file
        let metadata = cart::unpack_file(i_path, o_path, arc4key).unwrap_or_else(|err| {
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
        // Compile metadata from CLI
        let mut metadata = match params.jsonmeta {
            Some(j) => json::parse(&j).unwrap_or_else(|_| {
                println!("ERR: Invalid JSON metadata");
                process::exit(1);
            }),
            None => JsonValue::new_object(),
        };

        // Compile metadata from cartmeta file
        let m_path = format!("{}.cartmeta", params.file);
        let m_path = Path::new(&m_path);
        if m_path.is_file() {
            let contents = read_to_string(m_path).unwrap();
            for (k, v) in json::parse(&contents).unwrap().entries() {
                metadata.insert(k, v.to_string()).unwrap();
            }
        }

        // Only show metadata if requested
        if params.showmeta {
            println!("{}", metadata.pretty(4));
            process::exit(0);
        }

        // Assign provided filename to metadata if needed
        let name = params.name.unwrap_or(i_path.file_name().unwrap().to_string_lossy().to_string());
        metadata.insert("name", name).unwrap();

        // Generate and validate output path
        let o_path = params.outfile.unwrap_or(format!("{}.cart", params.file));
        let o_path = Path::new(&o_path);
        if o_path.is_file() && !params.force {
            eprintln!("ERR: File '{}' already exists", o_path.to_string_lossy());
            process::exit(1);
        }

        // Pack the file into CaRT format
        cart::pack_file(&i_path, &o_path, Some(metadata.dump()), None, arc4key).unwrap_or_else(|err| {
            eprintln!("ERR: Encountered error during packing ({})", err);
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

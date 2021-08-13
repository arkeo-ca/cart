mod globals;
mod cartobj;

use std::str;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write, Seek};
use json::JsonValue;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use crypto::sha2::Sha256;
use crypto::digest::Digest;

pub fn pack(mut istream: impl Read, mut ostream: impl Write, opt_header: Option<JsonValue>,
opt_footer: Option<JsonValue>, arc4_key: Option<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {

    let mut binary: Vec<u8> = Vec::new();
    istream.by_ref().read_to_end(&mut binary)?;

    let cart_obj = cartobj::CartObject::new(binary, arc4_key, opt_header, opt_footer, None)?;
    ostream.write_all(&cart_obj.pack()?[..])?;

    Ok(())
}

pub fn unpack(istream: impl Read+Seek, mut ostream: impl Write, arc4_key_override: Option<Vec<u8>>)
-> Result<(JsonValue, JsonValue), Box<dyn std::error::Error>> {
    let cart_obj = cartobj::CartObject::unpack(istream, arc4_key_override)?;
    ostream.write_all(&cart_obj.raw_binary()?)?;

    cart_obj.metadata()
}

pub fn examine(i_stream: impl Read+Seek, arc4_key_override: Option<Vec<u8>>)
-> Result<(JsonValue, JsonValue), Box<dyn std::error::Error>> {
    cartobj::CartObject::examine(i_stream, arc4_key_override)
}

pub fn pack_file(i_path: &Path, o_path: &Path, opt_header: Option<JsonValue>, opt_footer: Option<JsonValue>,
arc4_key_override: Option<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {

    let mut infile = File::open(i_path)?;
    let outfile = File::create(o_path)?;

    let mut binary: Vec<u8> = Vec::new();
    infile.read_to_end(&mut binary)?;

    // Generate default header metadata
    let mut header = opt_header.unwrap_or(JsonValue::new_object());
    if !header.has_key("name") {
        header.insert("name", i_path.file_name().unwrap().to_string_lossy().to_string())?;
    }

    // Generate default footer metadata
    let mut footer = opt_footer.unwrap_or(JsonValue::new_object());

    let mut md5_hasher = Md5::new();
    md5_hasher.input(&binary);
    let md5_digest = md5_hasher.result_str();

    let mut sha1_hasher = Sha1::new();
    sha1_hasher.input(&binary);
    let sha1_digest = sha1_hasher.result_str();

    let mut sha256_hasher = Sha256::new();
    sha256_hasher.input(&binary);
    let sha256_digest = sha256_hasher.result_str();

    footer.insert("length", binary.len().to_string())?;
    footer.insert("md5", md5_digest)?;
    footer.insert("sha1", sha1_digest)?;
    footer.insert("sha256", sha256_digest)?;

    pack(&binary[..], outfile, Some(header), Some(footer), arc4_key_override)?;

    Ok(())
}

pub fn unpack_file(i_path: &Path, o_path: &Path, arc4_key_override: Option<Vec<u8>>)
-> Result<(JsonValue, JsonValue), Box<dyn std::error::Error>> {

    let infile = File::open(i_path)?;
    let outfile = File::create(o_path)?;

    let metadata = unpack(infile, outfile, arc4_key_override)?;

    Ok(metadata)
}

pub fn examine_file(i_path: &Path, arc4_key_override: Option<Vec<u8>>)
-> Result<(JsonValue, JsonValue), Box<dyn std::error::Error>> {
    let infile = File::open(i_path)?;

    Ok(examine(infile, arc4_key_override)?)
}

pub fn is_cart(mut i_stream: impl Read) -> bool {
    let mut buffer = Vec::with_capacity(4);
    let _ = &mut i_stream.by_ref().take(4).read_to_end(&mut buffer);
    let magic = str::from_utf8(&buffer);

    match magic {
        Ok(m) => if m != globals::CART_MAGIC {return false}
        Err(_) => return false
    }

    let mut buffer = Vec::with_capacity(2);
    let _ = &mut i_stream.by_ref().take(2).read_to_end(&mut buffer);
    let version: i16 = bincode::deserialize(&buffer).unwrap_or(0);

    if version != globals::DEFAULT_VERSION {
        return false
    }

    true
}

pub fn is_cart_file(i_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let infile = File::open(i_path)?;

    Ok(is_cart(infile))
}

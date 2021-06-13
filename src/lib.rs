extern crate bincode;
extern crate crypto;
extern crate libflate;
extern crate json;

use serde::{Serialize, Deserialize};
use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::rc4::Rc4;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use libflate::zlib::{Encoder, EncodeOptions};
use json::object;
use std::io::{self, Read, Write};
use std::fs::File;
use std::path::Path;

const DEFAULT_VERSION: i16 = 1; // TODO Dynamically generate this constant from cargo package
const DEFAULT_ARC4_KEY: &[u8] = b"\x03\x01\x04\x01\x05\x09\x02\x06\x03\x01\x04\x01\x05\x09\x02\x06";
const CART_MAGIC: &str = "CART";
const TRAC_MAGIC: &str = "TRAC";
const BLOCK_SIZE: usize = 64 * 1024;


pub fn pack_stream(mut istream: impl Read, mut ostream: impl Write, opt_header: Option<String>,
opt_footer: Option<String>, arc4_key: Option<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {

    let mut binary: Vec<u8> = Vec::new();
    istream.by_ref().read_to_end(&mut binary)?;

    let mut footer = match opt_footer {
        Some(j) => json::parse(&j)?,
        None => object!(),
    };

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

    let cart_obj = CartObject::new(binary, arc4_key, opt_header, Some(footer.dump()), None)?;
    ostream.write_all(&cart_obj.pack()[..])?;

    Ok(())
}

pub fn unpack_stream(mut istream: impl Read, mut ostream: impl Write, arc4_key_override: Option<Vec<u8>>)
-> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn pack_file(i_path: &Path, o_path: &Path, opt_header: Option<String>, opt_footer: Option<String>,
arc4_key_override: Option<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {

    let infile = File::open(i_path)?;
    let outfile = File::create(o_path)?;

    // TODO Move default header generation out of here
    let mut header = match opt_header {
        Some(j) => json::parse(&j)?,
        None => object!(),
    };

    header.insert("name", i_path.file_name().unwrap().to_str())?;

    pack_stream(infile, outfile, Some(header.dump()), opt_footer, arc4_key_override)?;

    Ok(())
}

pub fn unpack_file(i_path: &Path, o_path: &Path, arc4_key_override: Option<Vec<u8>>)
-> Result<(), Box<dyn std::error::Error>> {

    let infile = File::open(i_path)?;
    let outfile = File::create(o_path)?;

    unpack_stream(infile, outfile, arc4_key_override)?;

    Ok(())
}

pub fn get_metadata_only(i_path: &str, arc4_key_override: Option<Vec<u8>>) {

}

pub fn is_cart(buffer: impl Read) -> bool {
    false
}

pub fn is_cart_file(i_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let infile = File::open(i_path)?;

    Ok(is_cart(infile))
}

#[derive(Serialize, Deserialize)]
struct CartObject {
    header: CartHeader,
    footer: CartFooter,
    binary: Vec<u8>,
}

impl CartObject {
    fn new(binary: Vec<u8>, arc4_key: Option<Vec<u8>>, opt_header: Option<String>,
    opt_footer: Option<String>, version: Option<i16>) -> Result<CartObject, Box<dyn std::error::Error>>{
        let arc4_key = match arc4_key {
            Some(k) => k,
            None => DEFAULT_ARC4_KEY.to_vec(),
        };

        let options = EncodeOptions::new().fixed_huffman_codes();
        let mut encoder = Encoder::with_options(Vec::new(), options).unwrap();
        io::copy(&mut binary.as_slice(), &mut encoder).unwrap();
        let deflated = encoder.finish().into_result().unwrap();

        let header = CartHeader::new(arc4_key, opt_header, version);
        let footer = CartFooter::new(opt_footer, 38 + header.opt_header.len() + deflated.len());

        Ok(CartObject{header, footer, binary: deflated})
    }

    fn pack(&self) -> Vec<u8> {
        let mut packed_cart: Vec<u8> = Vec::new();

        // Pack header
        packed_cart.extend(self.header.pack());

        // Pack binary
        let mut cipher = Rc4::new(&self.header.arc4_key);
        let mut out_binary: Vec<u8> = vec![0; self.binary.len()];
        cipher.process(&self.binary, &mut out_binary[..]);
        packed_cart.extend(out_binary);

        // Pack footer
        packed_cart.extend(self.footer.pack(&self.header.arc4_key));

        packed_cart
    }
}

#[derive(Serialize, Deserialize)]
struct CartHeader {
    version: i16,
    arc4_key: Vec<u8>,
    opt_header: String,
}

impl CartHeader {
    fn new(arc4_key: Vec<u8>, opt_header: Option<String>, version: Option<i16>) -> CartHeader {
        let version = match version {
            Some(k) => k,
            None => DEFAULT_VERSION,
        };

        let opt_header = match opt_header {
            Some(k) => k,
            None => String::from(""),
        };

        CartHeader{version, arc4_key, opt_header}
    }

    fn pack(&self) -> Vec<u8> {
        let mut packed_header: Vec<u8> = Vec::new();
        let opt_header_len = self.opt_header.len();

        // Pack mandatory header
        packed_header.extend(CART_MAGIC.as_bytes());
        packed_header.extend(bincode::serialize(&self.version).unwrap());
        packed_header.extend(bincode::serialize(&(0 as u64)).unwrap());
        if self.arc4_key == DEFAULT_ARC4_KEY.to_vec() {
            packed_header.extend(&self.arc4_key);
        } else {
            packed_header.extend(bincode::serialize(&(0 as u128)).unwrap());
        }
        packed_header.extend(bincode::serialize(&(opt_header_len as u64)).unwrap());

        // Pack optional header
        let mut cipher = Rc4::new(&self.arc4_key);
        let mut out_header: Vec<u8> = vec![0; opt_header_len];
        cipher.process(self.opt_header.as_bytes(), &mut out_header[..]);
        packed_header.extend(out_header);

        packed_header
    }
}

#[derive(Serialize, Deserialize)]
struct CartFooter {
    opt_footer: String,
    footer_pos: usize,
}

impl CartFooter {
    fn new(opt_footer: Option<String>, footer_pos: usize) -> CartFooter {
        let opt_footer = match opt_footer {
            Some(k) => k,
            None => String::from(""),
        };

        CartFooter{opt_footer, footer_pos}
    }

    fn pack(&self, arc4_key: &Vec<u8>) -> Vec<u8> {
        let mut packed_footer: Vec<u8> = Vec::new();
        let opt_footer_len = self.opt_footer.len();

        // Pack optional footer
        let mut cipher = Rc4::new(&arc4_key);
        let mut out_footer: Vec<u8> = vec![0; opt_footer_len];
        cipher.process(self.opt_footer.as_bytes(), &mut out_footer[..]);
        packed_footer.extend(out_footer);

        // Pack mandatory footer
        let opt_footer_pos = self.footer_pos;
        packed_footer.extend(TRAC_MAGIC.as_bytes());
        packed_footer.extend(bincode::serialize(&(0 as u64)).unwrap());
        packed_footer.extend(bincode::serialize(&(opt_footer_pos as u64)).unwrap());
        packed_footer.extend(bincode::serialize(&(opt_footer_len as u64)).unwrap());

        packed_footer
    }
}

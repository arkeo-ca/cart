extern crate bincode;
extern crate crypto;
extern crate libflate;
extern crate json;

use serde::{Serialize, Deserialize};
use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::rc4::Rc4;
use libflate::zlib::{Encoder, EncodeOptions};
use json::object;

use std::io::{self, Read, Write};
use std::fs::File;
use std::path::Path;

const DEFAULT_VERSION: i16 = 1; // TODO Dynamically generate this constant from cargo package
const DEFAULT_ARC4_KEY: &[u8] = b"\x03\x01\x04\x01\x05\x09\x02\x06\x03\x01\x04\x01\x05\x09\x02\x06";
const CART_MAGIC: &str = "CART";
const TRAC_MAGIC: &str = "TRAC";


pub fn pack_stream(mut istream: impl Read, mut ostream: impl Write, opt_header: Option<String>,
    opt_footer: Option<String>, arc4_key: Option<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {

    let mut binary: Vec<u8> = Vec::new();
    istream.by_ref().read_to_end(&mut binary)?;

    let cart_obj = CartObject::new(binary, arc4_key, opt_header, opt_footer, None)?;
    ostream.write_all(&cart_obj.pack()[..])?;

    Ok(())
}

pub fn unpack_stream(istream: impl Read, ostream: impl Write, arc4_key_override: Option<Vec<u8>>) {

}

pub fn pack_file(i_path: &Path, o_path: &Path, opt_header: Option<String>, opt_footer: Option<String>,
    arc4_key_override: Option<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {

    let infile = File::open(i_path)?;
    let outfile = File::create(o_path)?;

    let mut header = match opt_header {
        Some(k) => json::parse(&k)?,
        None => object!(),
    };

    header.insert("name", i_path.file_name().unwrap().to_str())?;

    pack_stream(infile, outfile, Some(header.dump()), opt_footer, arc4_key_override)?;

    Ok(())
}

pub fn unpack_file(i_path: &str, o_path: &str, arc4_key_override: Option<Vec<u8>>) {

}

pub fn get_metadata_only(i_path: &str, arc4_key_override: Option<Vec<u8>>) {

}

pub fn is_cart(buffer: impl Read) -> bool {
    false
}

#[derive(Serialize, Deserialize)]
struct CartObject {
    version: i16,
    arc4_key: Vec<u8>,
    opt_header: String,
    opt_footer: String,
    binary: Vec<u8>,
}

impl CartObject {
    fn new(binary: Vec<u8>, arc4_key: Option<Vec<u8>>, opt_header: Option<String>,
    opt_footer: Option<String>, version: Option<i16>) -> Result<CartObject, Box<dyn std::error::Error>>{
        let version = match version {
            Some(k) => k,
            None => DEFAULT_VERSION,
        };

        let arc4_key = match arc4_key {
            Some(k) => k,
            None => DEFAULT_ARC4_KEY.to_vec(),
        };

        let opt_header = match opt_header {
            Some(k) => k,
            None => String::from(""),
        };

        let opt_footer = match opt_footer {
            Some(k) => k,
            None => String::from(""),
        };

        Ok(CartObject{version, arc4_key, opt_header, opt_footer, binary})
    }

    fn pack_header(&self) -> Vec<u8> {
        let mut packed_header: Vec<u8> = Vec::new();
        let opt_header_len = self.opt_header.len();

        packed_header.extend(CART_MAGIC.as_bytes());
        packed_header.extend(bincode::serialize(&self.version).unwrap());
        packed_header.extend(bincode::serialize(&(0 as u64)).unwrap());

        if self.arc4_key == DEFAULT_ARC4_KEY.to_vec() {
            packed_header.extend(&self.arc4_key);
        } else {
            packed_header.extend(bincode::serialize(&(0 as u128)).unwrap());
        }

        packed_header.extend(bincode::serialize(&(opt_header_len as u64)).unwrap());

        let mut cipher = Rc4::new(&self.arc4_key);
        let mut out_header: Vec<u8> = vec![0; opt_header_len];
        cipher.process(self.opt_header.as_bytes(), &mut out_header[..]);
        packed_header.extend(out_header);

        packed_header
    }

    fn pack_binary(&self) -> Vec<u8> {
        let options = EncodeOptions::new().fixed_huffman_codes();
        let mut encoder = Encoder::with_options(Vec::new(), options).unwrap();
        io::copy(&mut self.binary.as_slice(), &mut encoder).unwrap();
        let inflated = encoder.finish().into_result().unwrap();

        let mut cipher = Rc4::new(&self.arc4_key);
        let mut out_binary: Vec<u8> = vec![0; inflated.len()];
        cipher.process(&inflated, &mut out_binary[..]);

        out_binary
    }

    fn pack_footer(&self) -> Vec<u8> {
        let mut packed_footer: Vec<u8> = Vec::new();
        let opt_footer_len = self.opt_footer.len();

        let mut cipher = Rc4::new(&self.arc4_key);
        let mut out_footer: Vec<u8> = vec![0; opt_footer_len];
        cipher.process(self.opt_footer.as_bytes(), &mut out_footer[..]);
        packed_footer.extend(out_footer);

        packed_footer.extend(TRAC_MAGIC.as_bytes());
        packed_footer.extend(bincode::serialize(&(0 as u128)).unwrap());
        packed_footer.extend(bincode::serialize(&(opt_footer_len as u64)).unwrap());

        packed_footer
    }

    fn pack(&self) -> Vec<u8> {
        let mut packed_cart: Vec<u8> = Vec::new();

        packed_cart.extend(self.pack_header());
        packed_cart.extend(self.pack_binary());
        packed_cart.extend(self.pack_footer());

        packed_cart
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_default_header() {
        let obj = CartObject::new(b"".to_vec(), None, None, None, None).unwrap();
        let packed = obj.pack_header();
        assert_eq!(b"\x43\x41\x52\x54\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x03\x01\x04\x01\x05\
        \x09\x02\x06\x03\x01\x04\x01\x05\x09\x02\x06\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), packed);
    }

    #[test]
    fn test_pack_header_with_metadata() {
        let opt_header = String::from("{\"name\":\"test.txt\"}");
        let obj = CartObject::new(b"".to_vec(), None, Some(opt_header), None, None).unwrap();
        let packed = obj.pack_header();
        assert_eq!(b"\x43\x41\x52\x54\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x03\x01\x04\x01\x05\
        \x09\x02\x06\x03\x01\x04\x01\x05\x09\x02\x06\x13\x00\x00\x00\x00\x00\x00\x00\xc2\xa4\xa5\
        \x5c\x53\xd5\x43\xf7\x79\x61\x33\xd7\x75\x1d\x94\xdd\xcb\xc4\xd4".to_vec(), packed);
    }

    #[test]
    fn test_pack_default_footer() {
        let obj = CartObject::new(b"".to_vec(), None, None, None, None).unwrap();
        let packed = obj.pack_footer();
        assert_eq!(b"\x54\x52\x41\x43\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
        \x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), packed);
    }

    #[test]
    fn test_pack_footer_with_metadata() {
        let opt_footer = String::from("{\"length\":\"5\"}");
        let obj = CartObject::new(b"".to_vec(), None, None, Some(opt_footer), None).unwrap();
        let packed = obj.pack_footer();
        assert_eq!(b"\xc2\xa4\xa7\x58\x50\xd7\x15\xa5\x79\x2f\x74\x91\x23\x4e\x54\x52\x41\x43\x00\
        \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x0e\x00\x00\x00\x00\x00\x00\
        \x00".to_vec(), packed);
    }
}



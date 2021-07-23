use crate::globals;

use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::rc4::Rc4;
use libflate::zlib::{Encoder, EncodeOptions, Decoder};
use std::io::{self, Read, Seek, SeekFrom};
use json::JsonValue;
use std::str;


pub struct CartObject {
    header: CartHeader,
    footer: CartFooter,
    binary: Vec<u8>,
}

impl CartObject {
    pub fn new(binary: Vec<u8>, arc4_key: Option<Vec<u8>>, opt_header: Option<JsonValue>,
    opt_footer: Option<JsonValue>, version: Option<i16>) -> Result<CartObject, Box<dyn std::error::Error>>{
        let arc4_key = arc4_key.unwrap_or(globals::DEFAULT_ARC4_KEY.to_vec());

        let options = EncodeOptions::new().fixed_huffman_codes();
        let mut encoder = Encoder::with_options(Vec::new(), options)?;
        io::copy(&mut binary.as_slice(), &mut encoder)?;
        let deflated = encoder.finish().into_result()?;

        let header = CartHeader::new(arc4_key, opt_header, version);
        let footer = CartFooter::new(opt_footer, 38 + header.opt_header.dump().len() + deflated.len());

        Ok(CartObject{header, footer, binary: deflated})
    }

    pub fn unpack(mut cart_stream: impl Read+Seek, arc4_key: Option<Vec<u8>>) -> Result<CartObject, Box<dyn std::error::Error>> {
        let header = CartHeader::unpack(&mut cart_stream, arc4_key)?;
        let footer = CartFooter::unpack(&mut cart_stream, &header.arc4_key)?;
        let binary = {
            let buffer_start = 38 + header.opt_header.len() as u64;
            let buffer_len = footer.opt_footer_pos as u64 - buffer_start;
            cart_stream.seek(SeekFrom::Start(buffer_start))?;

            let mut buffer = Vec::with_capacity(buffer_len as usize);
            let _ = cart_stream.by_ref().take(buffer_len).read_to_end(&mut buffer);

            let mut cipher = Rc4::new(&header.arc4_key);
            let mut plain_text: Vec<u8> = vec![0; buffer_len as usize];
            cipher.process(&buffer, &mut plain_text[..]);

            plain_text
        };

        Ok(CartObject{header, footer, binary})
    }

    pub fn pack(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut packed_cart: Vec<u8> = Vec::new();

        // Pack header
        packed_cart.extend(self.header.pack()?);

        // Pack binary
        let mut cipher = Rc4::new(&self.header.arc4_key);
        let mut out_binary: Vec<u8> = vec![0; self.binary.len()];
        cipher.process(&self.binary, &mut out_binary[..]);
        packed_cart.extend(out_binary);

        // Pack footer
        packed_cart.extend(self.footer.pack(&self.header.arc4_key)?);

        Ok(packed_cart)
    }

    pub fn raw_binary(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut decoder = Decoder::new(&self.binary[..])?;
        let mut inflated = Vec::new();
        decoder.read_to_end(&mut inflated)?;

        Ok(inflated)
    }

    pub fn metadata(&self) -> Result<JsonValue, Box<dyn std::error::Error>>{
        let mut metadata = self.header.opt_header.clone();
        let extra_metadata = &self.footer.opt_footer;

        for (k, v) in extra_metadata.entries() {
            metadata.insert(k, v.as_str())?;
        }
        Ok(metadata)
    }
}

struct CartHeader {
    magic: String,
    version: i16,
    arc4_key: Vec<u8>,
    opt_header: JsonValue,
}

impl CartHeader {
    fn new(arc4_key: Vec<u8>, opt_header: Option<JsonValue>, version: Option<i16>) -> CartHeader {
        let magic = String::from(globals::CART_MAGIC);
        let version = version.unwrap_or(globals::DEFAULT_VERSION);
        let opt_header = opt_header.unwrap_or(JsonValue::Null);

        CartHeader{magic, version, arc4_key, opt_header}
    }

    fn unpack(mut cart_stream: impl Read, arc4_key_override: Option<Vec<u8>>) -> Result<CartHeader, Box<dyn std::error::Error>> {
        // Unpack mandatory header
        let mut buffer = Vec::with_capacity(4);
        let _ = cart_stream.by_ref().take(4).read_to_end(&mut buffer);
        let magic = str::from_utf8(&buffer)?.to_string();

        let mut buffer = Vec::with_capacity(2);
        let _ = cart_stream.by_ref().take(2).read_to_end(&mut buffer);
        let version: i16 = bincode::deserialize(&buffer)?;

        let mut buffer = Vec::with_capacity(8);
        let _ = cart_stream.by_ref().take(8).read_to_end(&mut buffer);

        let mut buffer = Vec::with_capacity(16);
        let _ = cart_stream.by_ref().take(16).read_to_end(&mut buffer);
        let mut arc4_key = buffer.to_vec();

        if arc4_key == vec![0; 16] {
            arc4_key = arc4_key_override.unwrap_or(globals::DEFAULT_ARC4_KEY.to_vec());
        }

        let mut buffer = Vec::with_capacity(8);
        let _ = cart_stream.by_ref().take(8).read_to_end(&mut buffer);
        let opt_header_len: u64 = bincode::deserialize(&buffer)?;

        // Unpack optional header
        let mut buffer = Vec::with_capacity(opt_header_len as usize);
        let _ = cart_stream.by_ref().take(opt_header_len).read_to_end(&mut buffer);

        let mut cipher = Rc4::new(&arc4_key);
        let mut plain_text: Vec<u8> = vec![0; opt_header_len as usize];
        cipher.process(&buffer, &mut plain_text[..]);
        let opt_header = json::parse(&str::from_utf8(&plain_text)?.to_string())?;

        Ok(CartHeader{magic, version, arc4_key, opt_header})
    }

    fn pack(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut packed_header: Vec<u8> = Vec::new();
        let opt_header_str = self.opt_header.dump();

        // Pack mandatory header
        packed_header.extend(self.magic.as_bytes());
        packed_header.extend(bincode::serialize(&self.version)?);
        packed_header.extend(bincode::serialize(&(0 as u64))?);
        if self.arc4_key == globals::DEFAULT_ARC4_KEY.to_vec() {
            packed_header.extend(&self.arc4_key);
        } else {
            packed_header.extend(bincode::serialize(&(0 as u128))?);
        }
        packed_header.extend(bincode::serialize(&(opt_header_str.len() as u64))?);

        // Pack optional header
        let mut cipher = Rc4::new(&self.arc4_key);
        let mut out_header: Vec<u8> = vec![0; opt_header_str.len()];
        cipher.process(opt_header_str.as_bytes(), &mut out_header[..]);
        packed_header.extend(out_header);

        Ok(packed_header)
    }
}

struct CartFooter {
    opt_footer: JsonValue,
    opt_footer_pos: usize,
}

impl CartFooter {
    fn new(opt_footer: Option<JsonValue>, opt_footer_pos: usize) -> CartFooter {
        let opt_footer = opt_footer.unwrap_or(JsonValue::Null);

        CartFooter{opt_footer, opt_footer_pos}
    }

    fn unpack(mut cart_stream: impl Read+Seek, arc4_key: &Vec<u8>) -> Result<CartFooter, Box<dyn std::error::Error>> {
        // Unpack mandatory footer
        cart_stream.seek(SeekFrom::End(-28))?;

        let mut buffer = Vec::with_capacity(4);
        let _ = cart_stream.by_ref().take(4).read_to_end(&mut buffer);
        let _magic = str::from_utf8(&buffer)?.to_string();

        let mut buffer = Vec::with_capacity(8);
        let _ = cart_stream.by_ref().take(8).read_to_end(&mut buffer);

        let mut buffer = Vec::with_capacity(8);
        let _ = cart_stream.by_ref().take(8).read_to_end(&mut buffer);
        let opt_footer_pos: usize = bincode::deserialize(&buffer)?;

        let mut buffer = Vec::with_capacity(8);
        let _ = cart_stream.by_ref().take(8).read_to_end(&mut buffer);
        let opt_footer_len: u64 = bincode::deserialize(&buffer)?;

        // Unpack optional footer
        cart_stream.seek(SeekFrom::Start(opt_footer_pos as u64))?;

        let mut buffer = Vec::with_capacity(opt_footer_len as usize);
        let _ = cart_stream.by_ref().take(opt_footer_len).read_to_end(&mut buffer);

        let mut cipher = Rc4::new(&arc4_key);
        let mut plain_text: Vec<u8> = vec![0; opt_footer_len as usize];
        cipher.process(&buffer, &mut plain_text[..]);
        let opt_footer = json::parse(&str::from_utf8(&plain_text)?.to_string())?;

        Ok(CartFooter{opt_footer, opt_footer_pos})
    }

    fn pack(&self, arc4_key: &Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut packed_footer: Vec<u8> = Vec::new();
        let opt_footer_str = self.opt_footer.dump();

        // Pack optional footer
        let mut cipher = Rc4::new(&arc4_key);
        let mut out_footer: Vec<u8> = vec![0; opt_footer_str.len()];
        cipher.process(opt_footer_str.as_bytes(), &mut out_footer[..]);
        packed_footer.extend(out_footer);

        // Pack mandatory footer
        packed_footer.extend(globals::TRAC_MAGIC.as_bytes());
        packed_footer.extend(bincode::serialize(&(0 as u64))?);
        packed_footer.extend(bincode::serialize(&(self.opt_footer_pos as u64))?);
        packed_footer.extend(bincode::serialize(&(opt_footer_str.len() as u64))?);

        Ok(packed_footer)
    }
}

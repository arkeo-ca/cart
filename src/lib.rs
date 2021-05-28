extern crate bincode;
extern crate crypto;

use serde::{Serialize, Deserialize};
use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::rc4::Rc4;

const DEFAULT_VERSION: i16 = 1; // TODO Dynamically generate this constant from cargo package
//const RESERVED: u32 = 0;
const DEFAULT_ARC4_KEY: &[u8] = b"\x03\x01\x04\x01\x05\x09\x02\x06\x03\x01\x04\x01\x05\x09\x02\x06";
const CART_MAGIC: &str = "CART";
//const TRAC_MAGIC: &[u8; 4] = b"TRAC";
//const BLOCK_SIZE: u32 = 64 * 1024;



#[derive(Serialize, Deserialize)]
struct Header {
    magic: String,
    version: i16,
    reserved: u64,
    arc4_key: Vec<u8>,
    opt_header_len: u64,
    opt_header: String,
}

impl Header {
    fn new(version_override: Option<i16>, arc4_key_override: Option<Vec<u8>>, opt_header: Option<String>) -> Result<Header, &'static str>{
        let version = match version_override {
            Some(k) => k,
            None => DEFAULT_VERSION,
        };

        let arc4_key = match arc4_key_override {
            Some(k) => k,
            None => DEFAULT_ARC4_KEY.to_vec(),
        };

        let opt_header = match opt_header {
            Some(k) => k,
            None => String::from(""),
        };

        let opt_header_len = opt_header.len() as u64;

        Ok(Header{magic: String::from(CART_MAGIC), version, reserved:0, arc4_key, opt_header_len, opt_header})
    }

    fn pack(&self) -> Vec<u8> {
        let mut packed_header: Vec<u8> = Vec::with_capacity(38);
        packed_header.extend(self.magic.as_bytes());
        packed_header.extend(bincode::serialize(&self.version).unwrap());
        packed_header.extend(bincode::serialize(&self.reserved).unwrap());
        packed_header.extend(&self.arc4_key);
        packed_header.extend(bincode::serialize(&self.opt_header_len).unwrap());

        let mut cipher = Rc4::new(&self.arc4_key);
        let mut out_header: Vec<u8> = vec![0; self.opt_header_len as usize];
        cipher.process(self.opt_header.as_bytes(), &mut out_header[..]);

        packed_header.extend(out_header);
        packed_header
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_default_header() {
        let header = Header::new(None, None, None).unwrap();
        let packed = header.pack();
        assert_eq!(b"\x43\x41\x52\x54\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x03\x01\x04\x01\x05\x09\x02\x06\x03\x01\x04\x01\x05\x09\x02\x06\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), packed);
    }

    #[test]
    fn test_pack_header_with_metadata() {
        let opt_header = String::from("{\"name\":\"test.txt\"}");
        let header = Header::new(None, None, Some(opt_header)).unwrap();
        let packed = header.pack();
        assert_eq!(b"\x43\x41\x52\x54\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x03\x01\x04\x01\x05\x09\x02\x06\x03\x01\x04\x01\x05\x09\x02\x06\x13\x00\x00\x00\x00\x00\x00\x00\xc2\xa4\xa5\x5c\x53\xd5\x43\xf7\x79\x61\x33\xd7\x75\x1d\x94\xdd\xcb\xc4\xd4".to_vec(), packed);
    }
}



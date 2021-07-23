use json::{JsonValue, object};
use std::io::Cursor;

#[test]
fn test_empty() {
    let empty_stream: Vec<u8> = Vec::new();
    let mut cart_stream:Vec<u8> = Vec::new();
    let mut final_stream: Vec<u8> = Vec::new();

    cart::pack(&mut empty_stream.as_slice(), &mut cart_stream, None, None, None).unwrap();
    assert!(cart::is_cart(&cart_stream[..]));

    let mut cart_cur = Cursor::new(cart_stream);
    let (header, footer) = cart::unpack(&mut cart_cur, &mut final_stream, None).unwrap();
    assert_eq!(JsonValue::new_object(), header);
    assert_eq!(JsonValue::new_object(), footer);
    assert_eq!(empty_stream, final_stream);
}

#[test]
fn test_simple() {
    let small_stream: Vec<u8> = "This is a very bad file".as_bytes().to_vec();
    let mut cart_stream:Vec<u8> = Vec::new();
    let mut final_stream: Vec<u8> = Vec::new();

    let opt_header = object! {"testkey": "testvalue"};
    let opt_footer = object! {"complete": "yes"};

    cart::pack(&mut small_stream.as_slice(), &mut cart_stream, Some(opt_header.clone()), Some(opt_footer.clone()), None).unwrap();
    assert!(cart::is_cart(&cart_stream[..]));

    let mut cart_cur = Cursor::new(cart_stream);
    let (header, footer) = cart::unpack(&mut cart_cur, &mut final_stream, None).unwrap();
    assert_eq!(opt_header, header);
    assert_eq!(opt_footer, footer);
    assert_eq!(small_stream, final_stream);
}

#[test]
fn test_rc4_override() {
    let start_stream: Vec<u8> = "0123456789".as_bytes().to_vec();
    let mut cart_stream:Vec<u8> = Vec::new();
    let mut final_stream: Vec<u8> = Vec::new();

    let key = "Test Da Key!".as_bytes().to_vec();
    let opt_header = object! {"name": "testvalue"};
    let opt_footer = object! {"rc4key": "Test Da Key!"};

    cart::pack(&mut start_stream.as_slice(), &mut cart_stream, Some(opt_header.clone()), Some(opt_footer.clone()), Some(key.clone())).unwrap();
    assert!(cart::is_cart(&cart_stream[..]));

    let mut cart_cur = Cursor::new(cart_stream);
    let (header, footer) = cart::unpack(&mut cart_cur, &mut final_stream, Some(key)).unwrap();
    assert_eq!(opt_header, header);
    assert_eq!(opt_footer, footer);
    assert_eq!(start_stream, final_stream);
}

#[test]
fn test_not_a_cart() {
    let fake_stream: Vec<u8> = "0123456".as_bytes().to_vec();
    assert!(!cart::is_cart(&fake_stream[..]));
}

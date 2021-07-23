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
fn test_small() {
    let small_stream: Vec<u8> = vec![61]; // "="
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
fn test_large() {

}

#[test]
fn test_simple() {

}

#[test]
fn test_rc4_override() {

}

#[test]
fn test_not_a_cart() {

}

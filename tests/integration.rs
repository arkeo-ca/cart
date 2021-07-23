use json::JsonValue;
use std::io::Cursor;

#[test]
fn test_empty() {
    let empty_stream: Vec<u8> = Vec::new();
    let mut cart_stream:Vec<u8> = Vec::new();
    let mut final_stream: Vec<u8> = Vec::new();

    cart::pack(&mut empty_stream.as_slice(), &mut cart_stream, None, None, None).unwrap();
    assert!(cart::is_cart(&cart_stream[..]));

    let mut cart_cur = Cursor::new(cart_stream);
    let metadata = cart::unpack(&mut cart_cur, &mut final_stream, None);
    assert_eq!(JsonValue::Null, metadata.unwrap());
    assert_eq!(0, final_stream.len());
}

#[test]
fn test_small() {

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

#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate proptest;
extern crate stfu8;

use std::str;

fn assert_u8_round(v: &[u8]) {
    let encoded = stfu8::encode_u8(v);
    // validation, we may use from_utf8_unchecked in the future
    let _ = str::from_utf8(&encoded.as_bytes()).unwrap();
    let result = stfu8::decode_u8(&encoded).unwrap();
    assert_eq!(v, result.as_slice());
}

fn assert_u8_round_pretty(v: &[u8]) {
    let encoded = stfu8::encode_u8_pretty(v);
    // validation, we may use from_utf8_unchecked in the future
    let _ = str::from_utf8(&encoded.as_bytes()).unwrap();
    let result = stfu8::decode_u8(&encoded).unwrap();
    assert_eq!(v, result.as_slice());
}

proptest! {
    #[test]
    fn fuzz_u8_unicode(ref s in ".{0,300}") {
        assert_u8_round(&s.as_bytes());
        assert_u8_round_pretty(&s.as_bytes());
    }

    #[test]
    fn fuzz_u8_binary(ref v in proptest::collection::vec(0..256, 0..300)) {
        let v: Vec<u8> = v.iter().map(|i| *i as u8).collect();
        assert_u8_round(v.as_slice());
        assert_u8_round_pretty(v.as_slice());
    }
}

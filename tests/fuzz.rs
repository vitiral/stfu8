#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate proptest;
extern crate stfu8;

use std::str;
use std::u8;
use std::u16;
use std::u32;

const LEAD_MIN: u16 = 0xD800;
// const LEAD_MAX: u16 = 0xDBFF;
// const TRAIL_MIN: u16 = 0xDC00;
const TRAIL_MAX: u16 = 0xDFFF;

// U8 TESTS

fn assert_u8_round(v: &[u8]) {
    let encoded = stfu8::encode_u8(v);
    // validation, we may use from_utf8_unchecked in the future
    let _ = str::from_utf8(encoded.as_bytes()).unwrap();
    let result = stfu8::decode_u8(&encoded).unwrap();
    assert_eq!(v, result.as_slice());
}

fn assert_u8_round_pretty(v: &[u8]) {
    let encoded = stfu8::encode_u8_pretty(v);
    // validation, we may use from_utf8_unchecked in the future
    let _ = str::from_utf8(encoded.as_bytes()).unwrap();
    let result = stfu8::decode_u8(&encoded).unwrap();
    assert_eq!(v, result.as_slice());
}

fn assert_u16_round(v: &[u16]) {
    let encoded = stfu8::encode_u16(v);
    // validation, we may use from_utf8_unchecked in the future
    let _ = str::from_utf8(encoded.as_bytes()).unwrap();
    let result = stfu8::decode_u16(&encoded).unwrap();
    assert_eq!(v, result.as_slice());
}

fn assert_u16_round_pretty(v: &[u16]) {
    let encoded = stfu8::encode_u16_pretty(v);
    // validation, we may use from_utf8_unchecked in the future
    let _ = str::from_utf8(encoded.as_bytes()).unwrap();
    let result = stfu8::decode_u16(&encoded).unwrap();
    assert_eq!(v, result.as_slice());
}

proptest! {
    #[test]
    fn fuzz_all_unicode(ref s in ".{0,300}") {
        assert_u8_round(s.as_bytes());
        assert_u8_round_pretty(s.as_bytes());

        let utf16: Vec<u16> = s.encode_utf16().collect();
        assert_u16_round(&utf16);
        assert_u16_round_pretty(&utf16);
    }
}

proptest! {
    #[test]
    fn fuzz_u8_binary(ref v in proptest::collection::vec(0..256_u32, 0..300)) {
        let v: Vec<u8> = v.iter().map(|i| *i as u8).collect();
        assert_u8_round(v.as_slice());
        assert_u8_round_pretty(v.as_slice());

        let v16: Vec<u16> = v.iter().map(|c| u16::from(*c)).collect();
        assert_u16_round(v16.as_slice());
        assert_u16_round_pretty(v16.as_slice());
    }
}

proptest! {
    #[test]
    fn fuzz_u16_binary(ref v in proptest::collection::vec(0..(u32::from(u16::MAX) + 1), 0..300)) {
        let v: Vec<u16> = v.iter().map(|i| *i as u16).collect();
        assert_u16_round(v.as_slice());
        assert_u16_round_pretty(v.as_slice());
    }
}

proptest! {
    #[test]
    /// Fuzz test with ONLY ill-formed utf16 data
    fn fuzz_u32_illformed(ref v in proptest::collection::vec(LEAD_MIN..(TRAIL_MAX+1), 0..300)) {
        assert_u16_round(v.as_slice());
        assert_u16_round_pretty(v.as_slice());
    }
}

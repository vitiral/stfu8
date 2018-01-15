#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate proptest;
extern crate stfu8;

use proptest::prelude::*;

fn assert_round(v: &[u8]) {
    let result = stfu8::decode(&stfu8::encode(v)).unwrap();
    assert_eq!(v, result.as_slice());
}

fn assert_round_pretty(v: &[u8]) {
    let result = stfu8::decode(&stfu8::encode_pretty(v)).unwrap();
    assert_eq!(v, result.as_slice());
}

proptest! {
    #[test]
    fn fuzz_unicode(ref s in ".{0,300}") {
        assert_round(&s.as_bytes());
        assert_round_pretty(&s.as_bytes());
    }

    #[test]
    fn fuzz_binary(ref v in proptest::collection::vec(0..256, 0..300)) {
        let v: Vec<u8> = v.iter().map(|i| *i as u8).collect();
        assert_round(v.as_slice());
        assert_round_pretty(v.as_slice());
    }
}

#![allow(unknown_lints)]
#![allow(zero_width_space)]

#[macro_use]
extern crate pretty_assertions;
extern crate proptest;
extern crate stfu8;

use stfu8::{decode_u8, decode_u16, encode_u8, encode_u8_pretty, encode_u16, encode_u16_pretty};


use std::str;
use std::u16;

static SAMPLE_2_0: &str = include_str!("unicode-sample-2.0.txt");
static SAMPLE_3_2: &str = include_str!("unicode-sample-3.2.txt");
static SUPPLIMENTARY: &str = include_str!("unicode-supplimentary.txt");

// HELPERS

/// Do really basic stuff to make utf8 text into stfu8.
///
/// Note: purposefully not complete.
fn partial_encode(s: &str) -> String {
    s.replace("\\", r"\\")
}

/// Note: also tests u16, although not the edge cases
fn assert_round_u8(expected: &[u8]) {
    assert_eq!(
        expected,
        decode_u8(&encode_u8(expected)).unwrap().as_slice()
    );

    let utf16: Vec<u16> = expected.iter().map(|c| u16::from(*c)).collect();
    assert_eq!(
        utf16,
        decode_u16(&encode_u16(&utf16)).unwrap()
    );

}

fn assert_round_str(expected: &str) {
    assert_round_u8(expected.as_bytes());
}


fn assert_text(test: &str) {
    let expected = partial_encode(test);
    {
        let result = encode_u8_pretty(test.as_bytes());
        // validation, we may use from_utf8_unchecked in the future
        let _ = str::from_utf8(result.as_bytes()).unwrap();
        assert_eq!(expected, result);
        assert_eq!(test.as_bytes(), decode_u8(&result).unwrap().as_slice());
    }
    {
        let utf16: Vec<_> = test.encode_utf16().collect();
        let result = encode_u16_pretty(&utf16);
        let _ = str::from_utf8(result.as_bytes()).unwrap();
        assert_eq!(expected, result);
        assert_eq!(utf16.as_slice(), decode_u16(&result).unwrap().as_slice());
    }

}


#[test]
fn sanity_sample_2_0() {
    assert_text(SAMPLE_2_0);
}

#[test]
fn sanity_sample_3_2() {
    assert_text(SAMPLE_3_2);
}

#[test]
fn sanity_supplimentary() {
    assert_text(SUPPLIMENTARY);
}

#[test]
fn sanity_roundtrip() {
    assert_round_u8(b"");
    assert_round_u8(b"foo");
    assert_round_u8(b"\n");
    assert_round_u8(b"foo\n");
    assert_round_u8(b"\tfoo\n\tbar\n");
    assert_round_u8(b"\x0c\x22\xFE"); // note, some of the escaped are valid ascii
    assert_round_u8(b"\x0c\x22\xFE"); // note, some of the escaped are valid ascii
    assert_round_str("foo bar");
    assert_round_str("¡ ¢ £ ¤ ¥ ¦ § ¨ © ª « ¬ ­");
    assert_round_str(" ʰ ʱ ʲ ʳ ʴ ʵ ʶ ʷ ʸ ʹ ʺ ʻ");
    assert_round_str("܀ ܁ ܂ ܃ ܄ ܅ ܆ ܇ ܈ ܉ ܊ ܋ ܌ ܍ ܏");
    assert_round_str("Ꭰ Ꭱ Ꭲ Ꭳ Ꭴ Ꭵ Ꭶ Ꭷ Ꭸ Ꭹ");
    assert_round_str("ἀ ἁ ἂ ἃ ἄ ἅ ἆ ἇ Ἀ Ἁ");
    assert_round_str(
        "                          ​ ‌ ‍ ‎ ‏ ‐ ",
    );
    assert_round_str("‑ ‒ – — ― ‖ ‗ ‘ ’ ‚ ‛ “");
    assert_round_str("    ⃐ ⃑ ⃒ ⃓ ⃔ ⃕ ⃖ ⃗ ⃘ ⃙ ⃚ ⃛ ⃜ ⃝ ⃞ ⃟ ⃠ ⃡ ⃢ ⃣ ⃤ ⃥ ⃦ ⃧ ⃨ ⃩ ⃪ ");
}

// #[test]
// fn sanity_u8_decode() {
//     assert_eq!(
//         decode_u8(r"foo\u000072").unwrap(),
//         /*     */ b"foo\x72"
//     );
//
//     assert!(decode_u8(r"foo\u200172").is_err());
//     assert!(decode_u8(r"foo\foo").is_err());
//     assert!(decode_u8(r"foo\").is_err());
// }

#[test]
fn sanity_u8_decode() {
    assert_eq!(
        decode_u8(r"foo\u000072").unwrap(),
        /*     */ b"foo\x72"
    );

    assert_eq!(decode_u8(r"foo\u000156").unwrap(), "fooŖ".as_bytes());
    assert_eq!(decode_u8(r"foo\u02070E").unwrap(), "foo𠜎".as_bytes());
    assert!(decode_u8(r"foo\u220178").is_err());
    assert!(decode_u8(r"foo\u00D800").is_err()); // pair lead
    assert!(decode_u8(r"foo\foo").is_err());
    assert!(decode_u8(r"foo\").is_err());
}

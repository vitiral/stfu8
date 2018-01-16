#[macro_use]
extern crate pretty_assertions;
extern crate proptest;
extern crate rand;
extern crate regex_generate;
extern crate stfu8;

use stfu8::{decode_u8, encode_u8, encode_u8_pretty};


use std::str;

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

fn assert_round(expected: &[u8]) {
    assert_eq!(
        expected,
        decode_u8(&encode_u8(expected)).unwrap().as_slice()
    );
}

fn assert_round_str(expected: &str) {
    assert_round(expected.as_bytes());
}


// u8 tests

#[test]
fn sanity_u8_sample_2_0() {
    let expected = partial_encode(SAMPLE_2_0);
    let result = encode_u8_pretty(&SAMPLE_2_0.as_bytes());
    // validation, we may use from_utf8_unchecked in the future
    let _ = str::from_utf8(&result.as_bytes()).unwrap();
    assert_eq!(expected, result);
}

#[test]
fn sanity_u8_sample_3_2() {
    let expected = partial_encode(SAMPLE_3_2);
    let result = encode_u8_pretty(&SAMPLE_3_2.as_bytes());
    // validation, we may use from_utf8_unchecked in the future
    let _ = str::from_utf8(&result.as_bytes()).unwrap();
    assert_eq!(expected, result);
}

#[test]
fn sanity_u8_supplimentary() {
    // supplimentary character codes from http://www.i18nguy.com/unicode/supplementary-test.html
    let expected = partial_encode(SUPPLIMENTARY);
    let result = encode_u8_pretty(&SUPPLIMENTARY.as_bytes());
    // validation, we may use from_utf8_unchecked in the future
    let _ = str::from_utf8(&result.as_bytes()).unwrap();
    assert_eq!(expected, result);
}

#[test]
fn sanity_u8_roundtrip() {
    assert_round(b"");
    assert_round(b"foo");
    assert_round(b"\n");
    assert_round(b"foo\n");
    assert_round(b"\tfoo\n\tbar\n");
    assert_round(b"\x0c\x22\xFE"); // note, some of the escaped are valid ascii
    assert_round(b"\x0c\x22\xFE"); // note, some of the escaped are valid ascii
    assert_round_str("foo bar");
    assert_round_str("¡ ¢ £ ¤ ¥ ¦ § ¨ © ª « ¬ ­");
    assert_round_str(" ʰ ʱ ʲ ʳ ʴ ʵ ʶ ʷ ʸ ʹ ʺ ʻ");
    assert_round_str("܀ ܁ ܂ ܃ ܄ ܅ ܆ ܇ ܈ ܉ ܊ ܋ ܌ ܍ ܏");
    assert_round_str("Ꭰ Ꭱ Ꭲ Ꭳ Ꭴ Ꭵ Ꭶ Ꭷ Ꭸ Ꭹ");
    assert_round_str("ἀ ἁ ἂ ἃ ἄ ἅ ἆ ἇ Ἀ Ἁ");
    assert_round_str(
        "                          ​ ‌ ‍ ‎ ‏ ‐ ",
    );
    assert_round_str("‑ ‒ – — ― ‖ ‗ ‘ ’ ‚ ‛ “");;
    assert_round_str("    ⃐ ⃑ ⃒ ⃓ ⃔ ⃕ ⃖ ⃗ ⃘ ⃙ ⃚ ⃛ ⃜ ⃝ ⃞ ⃟ ⃠ ⃡ ⃢ ⃣ ⃤ ⃥ ⃦ ⃧ ⃨ ⃩ ⃪ ");
}

#[test]
fn sanity_u8_decode() {
    assert_eq!(
        decode_u8(r"foo\u000072").unwrap(),
        /*     */ b"foo\x72"
    );

    assert!(decode_u8(r"foo\u000172").is_err());
    assert!(decode_u8(r"foo\foo").is_err());
    assert!(decode_u8(r"foo\").is_err());
}


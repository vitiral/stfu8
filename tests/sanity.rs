#[macro_use]
extern crate pretty_assertions;
extern crate proptest;
extern crate rand;
extern crate regex_generate;

extern crate stfu8;

static SAMPLE_2_0: &str = include_str!("unicode-sample-2.0.txt");
static SAMPLE_3_2: &str = include_str!("unicode-sample-3.2.txt");

/// Do really basic stuff to make utf8 text into stfu8.
///
/// Note: purposefully not complete.
fn partial_encode(s: &str) -> String {
    s.replace("\\", r"\\")
}

#[test]
fn sanity_sample_2_0() {
    let expected = partial_encode(SAMPLE_2_0);
    let result = stfu8::encode_pretty(&SAMPLE_2_0.as_bytes());
    assert_eq!(expected, result);
}

#[test]
fn sanity_sample_3_2() {
    let expected = partial_encode(SAMPLE_3_2);
    let result = stfu8::encode_pretty(&SAMPLE_3_2.as_bytes());
    assert_eq!(expected, result);
}

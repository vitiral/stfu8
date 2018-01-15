/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 * Original Copyright 2012-2014 The Rust Project Developers
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! This code is practically copy/pasted from the rust std libraries'
//! `run_utf8_validation` function, used by `str::from_utf8`.

// FIXME: need to still work around some compiler intrinsics
#![allow(dead_code)]

use std::fmt;
use std::io::Write;

/*
Section: Creating a string
*/

/// Errors which can occur when attempting to interpret a sequence of [`u8`]
/// as a string.
///
/// [`u8`]: ../../std/primitive.u8.html
///
/// As such, the `from_utf8` family of functions and methods for both [`String`]s
/// and [`&str`]s make use of this error, for example.
///
/// [`String`]: ../../std/string/struct.String.html#method.from_utf8
/// [`&str`]: ../../std/str/fn.from_utf8.html
#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct Utf8Error {
    valid_up_to: usize,
    error_len: Option<u8>,
}

impl Utf8Error {
    /// Returns the index in the given string up to which valid UTF-8 was
    /// verified.
    ///
    /// It is the maximum index such that `from_utf8(&input[..index])`
    /// would return `Ok(_)`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::str;
    ///
    /// // some invalid bytes, in a vector
    /// let sparkle_heart = vec![0, 159, 146, 150];
    ///
    /// // std::str::from_utf8 returns a Utf8Error
    /// let error = str::from_utf8(&sparkle_heart).unwrap_err();
    ///
    /// // the second byte is invalid here
    /// assert_eq!(1, error.valid_up_to());
    /// ```
    pub fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }

    /// Provide more information about the failure:
    ///
    /// * `None`: the end of the input was reached unexpectedly.
    ///   `self.valid_up_to()` is 1 to 3 bytes from the end of the input.
    ///   If a byte stream (such as a file or a network socket) is being decoded incrementally,
    ///   this could be a valid `char` whose UTF-8 byte sequence is spanning multiple chunks.
    ///
    /// * `Some(len)`: an unexpected byte was encountered.
    ///   The length provided is that of the invalid byte sequence
    ///   that starts at the index given by `valid_up_to()`.
    ///   Decoding should resume after that sequence
    ///   (after inserting a U+FFFD REPLACEMENT CHARACTER) in case of lossy decoding.
    pub fn error_len(&self) -> Option<usize> {
        self.error_len.map(|len| len as usize)
    }
}

impl fmt::Display for Utf8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(error_len) = self.error_len {
            write!(
                f,
                "invalid utf-8 sequence of {} bytes from index {}",
                error_len, self.valid_up_to
            )
        } else {
            write!(
                f,
                "incomplete utf-8 byte sequence from index {}",
                self.valid_up_to
            )
        }
    }
}

/*
Section: UTF-8 validation
*/

// use truncation to fit u64 into usize
const NONASCII_MASK: usize = 0x80808080_80808080u64 as usize;

/// Returns `true` if any byte in the word `x` is nonascii (>= 128).
#[inline]
fn contains_nonascii(x: usize) -> bool {
    (x & NONASCII_MASK) != 0
}

/// Pretty much an exact copy of `run_utf8_validation` from the rust stdlib.
pub(crate) fn encode(encoder: &super::Encoder, v: &[u8]) -> String {
    let mut index = 0;
    let len = v.len();
    let mut out: Vec<u8> = Vec::with_capacity(len + len / 8);

    while index < len {
        let old_offset = index;
        debug_assert_eq!(b'\t', b'\x09');
        debug_assert_eq!(b'\n', b'\x0A');
        debug_assert_eq!(b'\r', b'\x0D');

        /// write a single byte that may be ascii.
        /// Escape it correctly no matter what.
        macro_rules! maybe_ascii { ($i: expr) => {{
            let b = v[$i];
            match b {
                b'\t' => if encoder.encode_tab {
                    out.extend_from_slice(b"\\t");
                } else {
                    out.push(b);
                },
                b'\n' => if encoder.encode_line_feed {
                    out.extend_from_slice(b"\\n");
                } else {
                    out.push(b);
                },
                b'\r' => if encoder.encode_cariage {
                    out.extend_from_slice(b"\\r");
                } else {
                    out.push(b);
                },
                b'\\' => out.extend_from_slice(b"\\\\"),
                0x20...0x7e => out.push(b), // visible ASCII
                _ => write!(out, r"\x{:0>2X}", b).unwrap(),
            }
        }}}

        /// Escape everything from old_offset to current index.
        /// It is invalid STFU-8 (which might be invalid utf8,
        /// or could just be the `\` character...)
        macro_rules! escape_them { () => {{
            for i in old_offset..(index+1) {
                maybe_ascii!(i);
            }
            index += 1;
            continue;
        }}}

        /// write everything from old_offset to current-index -- it
        /// is all valid utf8 and stfu8.
        macro_rules! write_them { () => {{
            for i in old_offset..(index+1) {
                out.push(v[i]);
            }
        }}}

        // original:
        // macro_rules! err {
        //     ($error_len: expr) => {
        //         return Err(Utf8Error {
        //             valid_up_to: old_offset,
        //             error_len: $error_len,
        //         })
        //     }
        // }

        macro_rules! next { () => {{
            index += 1;
            // we needed data, but there was none: error!
            if index >= len {
                index -= 1;   // added by me
                escape_them!(); // orig: err!(None)
            }
            v[index]
        }}}

        let first = v[index];
        if first >= 128 {
            let w = UTF8_CHAR_WIDTH[first as usize];
            // 2-byte encoding is for codepoints  \u{0080} to  \u{07ff}
            //        first  C2 80        last DF BF
            // 3-byte encoding is for codepoints  \u{0800} to  \u{ffff}
            //        first  E0 A0 80     last EF BF BF
            //   excluding surrogates codepoints  \u{d800} to  \u{dfff}
            //               ED A0 80 to       ED BF BF
            // 4-byte encoding is for codepoints \u{1000}0 to \u{10ff}ff
            //        first  F0 90 80 80  last F4 8F BF BF
            //
            // Use the UTF-8 syntax from the RFC
            //
            // https://tools.ietf.org/html/rfc3629
            // UTF8-1      = %x00-7F
            // UTF8-2      = %xC2-DF UTF8-tail
            // UTF8-3      = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2( UTF8-tail ) /
            //               %xED %x80-9F UTF8-tail / %xEE-EF 2( UTF8-tail )
            // UTF8-4      = %xF0 %x90-BF 2( UTF8-tail ) / %xF1-F3 3( UTF8-tail ) /
            //               %xF4 %x80-8F 2( UTF8-tail )
            match w {
                2 => {
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        escape_them!(); //orig: err!(Some(1))
                    }
                }
                3 => {
                    match (first, next!()) {
                        (0xE0, 0xA0...0xBF)
                        | (0xE1...0xEC, 0x80...0xBF)
                        | (0xED, 0x80...0x9F)
                        | (0xEE...0xEF, 0x80...0xBF) => {}
                        _ => escape_them!(), // orig: err!(Some(1))
                    }
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        escape_them!(); //orig: err!(Some(2))
                    }
                }
                4 => {
                    match (first, next!()) {
                        (0xF0, 0x90...0xBF) | (0xF1...0xF3, 0x80...0xBF) | (0xF4, 0x80...0x8F) => {}
                        _ => escape_them!(), //orig: err!(Some(1))
                    }
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        escape_them!(); //orig: err!(Some(2))
                    }
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        escape_them!(); //orig: err!(Some(3))
                    }
                }
                _ => escape_them!(), //orig: err!(Some(1))
            }
            // they were not invalid, so they are valid
            write_them!();
            index += 1;
        } else {
            // Ascii case
            maybe_ascii!(index);
            index += 1;
        }
    }
    String::from_utf8(out).expect("FIXME: this can be unchecked")
}

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
pub fn utf8_char_width(b: u8) -> usize {
    return UTF8_CHAR_WIDTH[b as usize] as usize;
}

/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;
/// Value of the tag bits (tag mask is !CONT_MASK) of a continuation byte.
const TAG_CONT_U8: u8 = 0b1000_0000;

#[test]
fn sanity_encode() {
    fn enc(s: &str) -> String {
        encode(&super::Encoder::new(), s.as_bytes())
    }
    fn assert_enc(s: &str) {
        assert_eq!(enc(s), s);
    }
    assert_enc("foo bar");
    assert_enc("¡ ¢ £ ¤ ¥ ¦ § ¨ © ª « ¬ ­");
    assert_enc(" ʰ ʱ ʲ ʳ ʴ ʵ ʶ ʷ ʸ ʹ ʺ ʻ");
    assert_enc("܀ ܁ ܂ ܃ ܄ ܅ ܆ ܇ ܈ ܉ ܊ ܋ ܌ ܍ ܏");
    assert_enc("Ꭰ Ꭱ Ꭲ Ꭳ Ꭴ Ꭵ Ꭶ Ꭷ Ꭸ Ꭹ");
    assert_enc("ἀ ἁ ἂ ἃ ἄ ἅ ἆ ἇ Ἀ Ἁ");
    assert_enc("                          ​ ‌ ‍ ‎ ‏ ‐ ");
    assert_enc("‑ ‒ – — ― ‖ ‗ ‘ ’ ‚ ‛ “");;
    assert_enc("    ⃐ ⃑ ⃒ ⃓ ⃔ ⃕ ⃖ ⃗ ⃘ ⃙ ⃚ ⃛ ⃜ ⃝ ⃞ ⃟ ⃠ ⃡ ⃢ ⃣ ⃤ ⃥ ⃦ ⃧ ⃨ ⃩ ⃪ ");

    // Test that `\` gets escaped
    assert_eq!(
        /**/ enc("¡ ¢ £ ¤ \\¥ ¦ § ¨ © ª « \\¬ ­"),
        /*   */ r"¡ ¢ £ ¤ \\¥ ¦ § ¨ © ª « \\¬ ­"
    );

    // Test that newlines gets escaped
    assert_eq!(
        /**/ enc("Ā ā Ă \nă Ą ą Ć\n ć Ĉ ĉ\n"),
        /*   */ r"Ā ā Ă \nă Ą ą Ć\n ć Ĉ ĉ\n",
    );
}

#[test]
fn sanity_encode_binary() {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice("¡ ¢ £".as_bytes());
    bytes.extend_from_slice(b"\t\n\r"); // "\x09\x0a\x0d"
    bytes.extend_from_slice(b"\x07\x7f\xFE");
    bytes.extend_from_slice("¤ ¥ ¦".as_bytes());
    assert_eq!(encode(&super::Encoder::new(), &bytes), r"¡ ¢ £\t\n\r\x07\x7F\xFE¤ ¥ ¦");
}

#[test]
fn sanity_encode_pretty() {
    let expected = "foo\nbar\n";
    let result = encode(&super::Encoder::pretty(), expected.as_bytes());
    assert_eq!(expected, result);
}

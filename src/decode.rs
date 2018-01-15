/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use regex::Regex;
use std::char;
use std::error::Error;
use std::fmt;

lazy_static! {
    static ref ESCAPED_RE: Regex = Regex::new(
        r#"(?x)
        \\t|                # repr tab
        \\n|                # repr newline
        \\r|                # repr linefeed
        \\\\|               # repr backslash
        \\x[0-9a-fA-F]{2}|  # repr hex-byte
        \\                  # INVALID
        "#).unwrap();
}

const INDEX_TAB: usize = 1;
const INDEX_NEWLINE: usize = 2;
const INDEX_LINEFEED: usize = 3;
const INDEX_BACKSLASH: usize = 4;
const INDEX_HEX: usize = 5;
const INDEX_INVALID: usize = 6;

#[derive(Debug)]
pub struct DecodeError {
    pub index: usize,
}

/// Decode a utf8 string containing encoded STFU-8 into binary.
///
/// # Examples
/// ```rust
/// # extern crate stfu8;
///
/// # fn main() {
/// let expected = b"foo\xFF\nbar";
/// let encoded = stfu8::encode_pretty(expected);
/// assert_eq!(
///     expected,
///     stfu8::decode(&encoded).unwrap().as_slice()
/// );
/// # }
/// ```
pub fn decode(s: &str) -> Result<Vec<u8>, DecodeError> {
    // keep track of the last index observed
    let mut last_end = 0;
    let mut out = Vec::new();
    let v = s.as_bytes();
    for mat in ESCAPED_RE.find_iter(s) {
        let start = mat.start();
        // push bytes that didn't need to be escaped
        out.extend_from_slice(&v[last_end..start]);
        match mat.as_str() {
            "\\t" => out.push(b'\t'),
            "\\n" => out.push(b'\n'),
            "\\r" => out.push(b'\r'),
            "\\\\" => out.push(b'\\'),
            "\\" => return Err(DecodeError { index: start }),
            hex => {
                let hex = hex.as_bytes();
                debug_assert_eq!(4, hex.len());
                debug_assert_eq!(b'\\', hex[0]);
                debug_assert_eq!(b'x', hex[1]);
                let byte: u8 = (from_hex(hex[2]) << 4) + from_hex(hex[3]);
                out.push(byte);
            }
        }
        last_end = mat.end();
    }
    let len = v.len();
    out.extend_from_slice(&v[last_end..len]);
    Ok(out)
}

#[inline(always)]
/// Convert a hexidecimal character (`0-F`) into it's corresponding numerical value (0-15)
fn from_hex(b: u8) -> u8 {
    (b as char).to_digit(16).unwrap() as u8
}

impl Error for DecodeError {
    fn description(&self) -> &str {
        "failure decoding as STFU8"
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed decoding, found invalid byte at index={}",
               self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::encode;

    fn assert_round(expected: &[u8]) {
        assert_eq!(
            expected,
            decode(&encode(expected)).unwrap().as_slice()
        );
    }

    fn assert_round_str(expected: &str) {
        assert_round(expected.as_bytes());
    }

    #[test]
    fn sanity_roundtrip() {
        assert_round(b"");
        assert_round(b"foo");
        assert_round(b"\n");
        assert_round(b"foo\n");
        assert_round(b"\tfoo\n\tbar\n");
        assert_round(b"\x0c\x22\xFE");  // note, some of the escaped are valid ascii
        assert_round(b"\x0c\x22\xFE");  // note, some of the escaped are valid ascii
        assert_round_str("foo bar");
        assert_round_str("¡ ¢ £ ¤ ¥ ¦ § ¨ © ª « ¬ ­");
        assert_round_str(" ʰ ʱ ʲ ʳ ʴ ʵ ʶ ʷ ʸ ʹ ʺ ʻ");
        assert_round_str("܀ ܁ ܂ ܃ ܄ ܅ ܆ ܇ ܈ ܉ ܊ ܋ ܌ ܍ ܏");
        assert_round_str("Ꭰ Ꭱ Ꭲ Ꭳ Ꭴ Ꭵ Ꭶ Ꭷ Ꭸ Ꭹ");
        assert_round_str("ἀ ἁ ἂ ἃ ἄ ἅ ἆ ἇ Ἀ Ἁ");
        assert_round_str("                          ​ ‌ ‍ ‎ ‏ ‐ ");
        assert_round_str("‑ ‒ – — ― ‖ ‗ ‘ ’ ‚ ‛ “");;
        assert_round_str("    ⃐ ⃑ ⃒ ⃓ ⃔ ⃕ ⃖ ⃗ ⃘ ⃙ ⃚ ⃛ ⃜ ⃝ ⃞ ⃟ ⃠ ⃡ ⃢ ⃣ ⃤ ⃥ ⃦ ⃧ ⃨ ⃩ ⃪ ");
    }
}

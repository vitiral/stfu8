/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use regex::Regex;
use std::error::Error;
use std::fmt;

use helpers::from_hex2;

lazy_static! {
    static ref ESCAPED_RE: Regex = Regex::new(
        r#"(?x)
        \\t|                # repr tab
        \\n|                # repr newline
        \\r|                # repr linefeed
        \\\\|               # repr backslash
        \\x[0-9a-fA-F]{2}|  # repr hex-byte
        \\u[0-9a-fA-F]{6}|  # repr code point
        \\                  # INVALID
        "#).unwrap();
}

#[derive(Debug)]
pub struct DecodeError {
    pub index: usize,
}

/// Decode a UTF-8 string containing encoded STFU-8 into binary.
///
/// # Examples
/// ```rust
/// # extern crate stfu8;
///
/// # fn main() {
/// let expected = b"foo\xFF\nbar";
/// let encoded = stfu8::encode_u8_pretty(expected);
/// assert_eq!(
///     encoded,
///     "foo\\xFF\nbar"
/// );
/// assert_eq!(
///     expected,
///     stfu8::decode_u8(&encoded).unwrap().as_slice()
/// );
/// # }
/// ```
pub fn decode_u8(s: &str) -> Result<Vec<u8>, DecodeError> {
    // keep track of the last index observed
    let mut last_end = 0;
    let mut out = Vec::new();
    let v = s.as_bytes();
    for mat in ESCAPED_RE.find_iter(s) {
        let start = mat.start();
        // push bytes that didn't need to be escaped
        out.extend_from_slice(&v[last_end..start]);
        if mat.as_str() == "\\" {
            return Err(DecodeError { index: start });
        }

        match &mat.as_str()[..2] {
            "\\t" => out.push(b'\t'),
            "\\n" => out.push(b'\n'),
            "\\r" => out.push(b'\r'),
            "\\\\" => out.push(b'\\'),
            "\\x" => out.push(from_hex2(&mat.as_str().as_bytes()[2..])),
            "\\u" => {
                // it will handle \u even though the roundtrip will be invalid.
                let hex6 = &mat.as_str().as_bytes()[2..];
                debug_assert_eq!(6, hex6.len());
                out.push(0); // \u is used to specify a u32 character
                out.push(from_hex2(&hex6[0..2]));
                out.push(from_hex2(&hex6[2..4]));
                out.push(from_hex2(&hex6[4..]));
            }
            _ => unreachable!("disallowed by regex"),
        }
        last_end = mat.end();
    }
    let len = v.len();
    out.extend_from_slice(&v[last_end..len]);
    Ok(out)
}

impl Error for DecodeError {
    fn description(&self) -> &str {
        "failure decoding as STFU8"
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Failed decoding, found invalid byte at index={}",
            self.index
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::encode_u8;

    fn assert_round(expected: &[u8]) {
        assert_eq!(
            expected,
            decode_u8(&encode_u8(expected)).unwrap().as_slice()
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
    fn sanity_code_point() {
        assert_eq!(
            decode_u8(r"foo\u00f372").unwrap(),
            /*  */ b"foo\x00\x00\xf3\x72"
        );
    }
}

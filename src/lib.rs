/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! # STFU-8: Sorta Text Format in UTF-8
//! Basically STFU-8 is the text format you already write when use escape codes in C, python, rust,
//! etc.
//!
//! It permits binary data in UTF-8 by escaping them with `\`, for instance `\n` and `\x0F`.
//!
//! See the documentation for:
//! - [`encode_u8`](fn.encode_u8.html) and [`decode_u8`](fn.decode_u8.html)
//! - [`encode_u16`](fn.encode_u16.html) and [`decode_u16`](fn.decode_u16.html)
//!
//! Also consider [starring the project on github](https://github.com/vitiral/stfu8)
#[macro_use]
extern crate lazy_static;
extern crate regex;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use std::u8;
use std::u16;

mod helpers;
mod decode;
mod encode_u8;
mod encode_u16;

pub use decode::{DecodeError, DecodeErrorKind};

/// Encode text as STFU-8, escaping all non-printable characters.
///
/// > Also check out [`encode_u8_pretty`](fn.encode_u8_pretty.html)
///
/// # Examples
/// ```rust
/// # extern crate stfu8;
///
/// # fn main() {
/// let encoded = stfu8::encode_u8(b"foo\xFF\nbar");
/// assert_eq!(
///     encoded,
///     r"foo\xFF\nbar" // notice the `r` == raw string
/// );
/// # }
/// ```
pub fn encode_u8(v: &[u8]) -> String {
    let encoder = Encoder::new();
    encode_u8::encode(&encoder, v)
}

/// Decode STFU-8 text as binary, escaping all non-printable characters EXCEPT:
/// - `\t`: tab
/// - `\n`: line feed
/// - `\r`: cariage return
///
/// This will allow the encoded text to print "pretilly" while still escaping invalid unicode and
/// other non-printable characters.
///
/// > Also check out [`encode_u8`](fn.encode_u8.html)
///
/// # Examples
/// ```rust
/// # extern crate stfu8;
///
/// # fn main() {
/// let encoded = stfu8::encode_u8_pretty(b"foo\xFF\nbar");
/// assert_eq!(
///     encoded,
///     "foo\\xFF\nbar"
/// );
/// # }
/// ```
pub fn encode_u8_pretty(v: &[u8]) -> String {
    let encoder = Encoder::pretty();
    encode_u8::encode(&encoder, v)
}

/// Encode text as STFU-8, escaping all non-printable characters.
///
/// > Also check out [`encode_u16_pretty`](fn.encode_u16_pretty.html)
///
/// # Examples
/// ```rust
/// # extern crate stfu8;
///
/// # fn main() {
/// let mut ill: Vec<u16> = b"foo\xFF\nbar"
///     .iter()
///     .map(|b| *b as u16)
///     .collect();
///
/// // Make it ill formed UTF-16
/// ill.push(0xD800);       // surrogate pair lead
/// ill.push(b' ' as u16);  // NOT a trail
/// ill.push(0xDEED);       // Trail... with no lead
/// ill.push(b' ' as u16);
/// ill.push(0xDABA);       // lead... but end of str
/// let encoded = stfu8::encode_u16(ill.as_slice());
///
/// // Note that 0xFF is the valid character "ÿ"
/// // and the ill-formed characters are escaped.
/// assert_eq!(
///     encoded,
///     r"fooÿ\nbar\u00D800 \u00DEED \u00DABA"
/// );
/// # }
/// ```
pub fn encode_u16(v: &[u16]) -> String {
    let encoder = Encoder::new();
    encode_u16::encode(&encoder, v)
}

/// Decode STFU-8 text as binary, escaping all non-printable characters EXCEPT:
/// - `\t`: tab
/// - `\n`: line feed
/// - `\r`: cariage return
///
/// This will allow the encoded text to print "pretilly" while still escaping invalid unicode and
/// other non-printable characters.
///
/// > Also check out [`encode_u16`](fn.encode_u16.html)
///
/// # Examples
/// ```rust
/// # extern crate stfu8;
///
/// # fn main() {
/// let mut ill: Vec<u16> = b"foo\xFF\nbar"
///     .iter()
///     .map(|b| *b as u16)
///     .collect();
///
/// // Make it ill formed UTF-16
/// ill.push(0xD800);       // surrogate pair lead
/// ill.push(b' ' as u16);  // NOT a trail
/// ill.push(0xDEED);       // Trail... with no lead
/// ill.push(b' ' as u16);
/// ill.push(0xDABA);       // lead... but end of str
/// let encoded = stfu8::encode_u16_pretty(ill.as_slice());
///
/// // Note that 0xFF is the valid character "ÿ"
/// // and the ill-formed characters are escaped.
/// assert_eq!(
///     encoded,
///     "fooÿ\nbar\\u00D800 \\u00DEED \\u00DABA"
/// );
/// # }
/// ```
pub fn encode_u16_pretty(v: &[u16]) -> String {
    let encoder = Encoder::pretty();
    encode_u16::encode(&encoder, v)
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
    let mut out: Vec<u8> = Vec::new();
    {
        let f = |val: decode::PushGeneric| -> Result<(), DecodeError> {
            match val {
                decode::PushGeneric::Value{val, start} => {
                    if val > u8::MAX as u32 {
                        Err(DecodeError {
                            index: start,
                            kind: DecodeErrorKind::InvalidValue,
                        })
                    } else {
                        out.push(val as u8);
                        Ok(())
                    }
                },
                decode::PushGeneric::String(s) => {
                    out.extend_from_slice(&s.as_bytes());
                    Ok(())
                }
            }
        };
        decode::decode_generic(f, s)?;
    }
    Ok(out)
}

/// Decode a UTF-8 string containing encoded STFU-8 into a `Vec<u16>`.
///
/// # Examples
/// ```rust
/// # extern crate stfu8;
///
/// # fn main() {
/// let mut ill: Vec<u16> = b"foo\xFF\nbar"
///     .iter()
///     .map(|b| *b as u16)
///     .collect();
///
/// // Make it ill formed UTF-16
/// ill.push(0xD800);       // surrogate pair lead
/// ill.push(b' ' as u16);  // NOT a trail
/// ill.push(0xDEED);       // Trail... with no lead
/// ill.push(b' ' as u16);
/// ill.push(0xDABA);       // lead... but end of str
/// let encoded = stfu8::encode_u16(ill.as_slice());
///
/// // Note that 0xFF is the valid character "ÿ"
/// // and the ill-formed characters are escaped.
/// assert_eq!(
///     encoded,
///     r"fooÿ\nbar\u00D800 \u00DEED \u00DABA"
/// );
///
/// assert_eq!(ill, stfu8::decode_u16(&encoded).unwrap());
/// # }
/// ```
pub fn decode_u16(s: &str) -> Result<Vec<u16>, DecodeError> {
    let mut out: Vec<u16> = Vec::new();
    {
        let f = |val: decode::PushGeneric| -> Result<(), DecodeError> {
            match val {
                decode::PushGeneric::Value{val, start} => {
                    if val > u16::MAX as u32 {
                        Err(DecodeError {
                            index: start,
                            kind: DecodeErrorKind::InvalidValue,
                        })
                    } else {
                        out.push(val as u16);
                        Ok(())
                    }
                },
                decode::PushGeneric::String(s) => {
                    for c in s.chars() {
                        let mut buf = [0u16; 2];
                        out.extend_from_slice(helpers::to_utf16(c, &mut buf));
                    }
                    Ok(())
                }
            }
        };
        decode::decode_generic(f, s)?;
    }
    Ok(out)
}


// NOT YET STABILIZED

/// Settings for encoding binary data.
///
/// TODO: make this public eventually
pub(crate) struct Encoder {
    pub(crate) encode_tab: bool,       // \t \x09
    pub(crate) encode_line_feed: bool, // \n \x0A
    pub(crate) encode_cariage: bool,   // \r \x0D
}

impl Encoder {
    /// Create a new "non pretty" `Encoder`.
    ///
    /// ALL non-printable characters will be escaped
    pub fn new() -> Encoder {
        Encoder {
            encode_tab: true,
            encode_line_feed: true,
            encode_cariage: true,
        }
    }

    /// Create a "pretty" `Encoder`.
    ///
    /// The following non-printable characters will not be escaped:
    /// - `\t`: tab
    /// - `\n`: line feed
    /// - `\r`: cariage return
    pub fn pretty() -> Encoder {
        Encoder {
            encode_tab: false,
            encode_line_feed: false,
            encode_cariage: false,
        }
    }
}

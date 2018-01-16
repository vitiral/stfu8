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
//! See the documentation for [`encode_u8`](fn.encode_u8.html) and [`decode_u8`](fn.decode_u8.html)
//! for how to use the library.
//!
//! Also consider [starring the project on github](https://github.com/vitiral/stfu8)
#[macro_use]
extern crate lazy_static;
extern crate regex;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod helpers;
mod decode;
mod encode_u8;
// mod encode_u32;

pub use decode::{decode_u8, DecodeError};

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

// NOT YET STABILIZED

/// Settings for encoding binary data.
///
/// TODO: make this public eventually
pub(crate) struct Encoder {
    pub(crate) encode_tab: bool,          // \t \x09
    pub(crate) encode_line_feed: bool,    // \n \x0A
    pub(crate) encode_cariage: bool,      // \r \x0D
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

/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use std::char;
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
pub enum DecodeErrorKind {
    UnescapedSlash,
    InvalidValue,
}

#[derive(Debug)]
pub struct DecodeError {
    pub kind: DecodeErrorKind,
    pub index: usize,
    pub(crate) mat: String,
}

pub(crate) enum PushGeneric<'a> {
    /// Push a value that may be invalid.
    Value { start: usize, val: u32 },
    /// Push an always-valid string.
    String(&'a str),
}

/// Decode generically
pub(crate) fn decode_generic<'a, F>(
    mut push_val: F,
    s: &'a str
)
-> Result<(), DecodeError>
    where F: FnMut(PushGeneric) -> Result<(), DecodeError>
{
    // keep track of the last index observed
    let mut last_end = 0;
    for mat in ESCAPED_RE.find_iter(s) {
        let start = mat.start();
        // push bytes that didn't need to be escaped
        push_val(PushGeneric::String(&s[last_end..start]))?;
        if mat.as_str() == "\\" {
            return Err(DecodeError {
                index: start,
                kind: DecodeErrorKind::UnescapedSlash,
                mat: mat.as_str().to_string(),
            })
        }

        let c32 = match &mat.as_str()[..2] {
            "\\t" => b'\t' as u32,
            "\\n" => b'\n' as u32,
            "\\r" => b'\r' as u32,
            "\\\\" =>b'\\' as u32,
            "\\x" => from_hex2(&mat.as_str().as_bytes()[2..]) as u32,
            "\\u" => {
                let hex6 = &mat.as_str().as_bytes()[2..];
                debug_assert_eq!(6, hex6.len());
                let d0 = from_hex2(&hex6[0..2]) as u32;
                let d1 = from_hex2(&hex6[2..4]) as u32;
                let d2 = from_hex2(&hex6[4..]) as u32;

                let c32 = (d0 << 16) + (d1 << 8) + d2;
                match char::from_u32(c32) {
                    Some(c) => {
                        // It is a valid UTF code point. Always
                        // decode it as such.
                        let mut out = String::with_capacity(4);
                        out.push(c);
                        push_val(PushGeneric::String(&out))?;
                        last_end = mat.end();
                        continue;
                    },
                    // It is not a valid code point. Still try
                    // to record it's value "as is".
                    None => c32,
                }
            }
            _ => unreachable!("disallowed by regex"),
        };
        push_val(PushGeneric::Value {start: mat.start(), val: c32 })?;
        last_end = mat.end();
    }
    let len = s.len();
    push_val(PushGeneric::String(&s[last_end..len]))?;
    Ok(())
}

impl Error for DecodeError {
    fn description(&self) -> &str {
        match self.kind {
            DecodeErrorKind::UnescapedSlash => r#"Found unmatched '\'. Use "\\" to escape slashes"#,
            DecodeErrorKind::InvalidValue => r#"Escaped value is out of range of the decoder"#,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} when decoding {:?} [index={}]", self.index, self, self.mat)
    }
}

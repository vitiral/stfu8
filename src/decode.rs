/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use std::char;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum DecodeErrorKind {
    /// A single unescaped backslash was found. Either the following character doesn't
    /// start a valid escape sequence or it is at the end of the string.
    UnescapedSlash,
    /// The value from a '\x' or '\u' hexadecimal escape sequence is out of range for the decode.
    InvalidValue,
    /// There are not enough characters after a '\x' or '\u' to build a escape sequence.
    HexNumberToShort,
    /// The required characters after a '\x' or '\u' are not all valid hex digits.
    InvalidHexDigit,
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
pub(crate) fn decode_generic<F>(mut push_val: F, s: &str) -> Result<(), DecodeError>
where
    F: FnMut(PushGeneric) -> Result<(), DecodeError>,
{
    let mut string = s;
    let mut offset = 0;

    while let Some(byte_index) = string.find('\\') {
        if byte_index > 0 {
            push_val(PushGeneric::String(&string[..byte_index]))?;
        }
        // byte index of the backslash in the original string
        let start_idx = offset + byte_index;
        let rest = string.len() - byte_index;
        if rest < 2 {
            Err(DecodeError {
                index: start_idx,
                kind: DecodeErrorKind::UnescapedSlash,
                mat: string[byte_index..].to_string(),
            })?
        }

        // macro to create a PushGeneric::Value
        macro_rules! pg_value {
            ( $v:expr ) => {{
                PushGeneric::Value {
                    start: start_idx,
                    val: $v as u32,
                }
            }};
        }
        let consumed_bytes = match &string.as_bytes()[byte_index + 1] {
            b't' => {
                push_val(pg_value!(b'\t'))?;
                2
            }
            b'n' => {
                push_val(pg_value!(b'\n'))?;
                2
            }
            b'r' => {
                push_val(pg_value!(b'\r'))?;
                2
            }
            b'\\' => {
                push_val(pg_value!(b'\\'))?;
                2
            }
            b'x' => {
                if rest < 4 {
                    Err(DecodeError {
                        index: start_idx,
                        kind: DecodeErrorKind::HexNumberToShort,
                        mat: string[byte_index..].to_string(),
                    })?
                }

                match u32::from_str_radix(&string[(byte_index + 2)..(byte_index + 4)], 16) {
                    Ok(x) => push_val(pg_value!(x)),
                    Err(_) => Err(DecodeError {
                        index: start_idx,
                        kind: DecodeErrorKind::InvalidHexDigit,
                        mat: s.to_string(),
                    }),
                }?;
                4
            }
            b'u' => {
                if rest < 8 {
                    Err(DecodeError {
                        index: start_idx,
                        kind: DecodeErrorKind::HexNumberToShort,
                        mat: string[byte_index..].to_string(),
                    })?
                }

                let c32 = match u32::from_str_radix(&string[(byte_index + 2)..(byte_index + 8)], 16)
                {
                    Ok(x) => Ok(x),
                    Err(_) => Err(DecodeError {
                        index: start_idx,
                        kind: DecodeErrorKind::InvalidHexDigit,
                        mat: s.to_string(),
                    }),
                }?;

                match char::from_u32(c32) {
                    // It is a valid UTF code point. Always
                    // decode it as such.
                    Some(c) => push_val(PushGeneric::String(&c.to_string())),
                    // It is not a valid code point. Still try
                    // to record it's value "as is".
                    None => push_val(pg_value!(c32)),
                }?;
                8
            }
            _ => Err(DecodeError {
                index: start_idx,
                kind: DecodeErrorKind::UnescapedSlash,
                mat: string[byte_index..].to_string(),
            })?,
        };

        string = &string[(byte_index + consumed_bytes)..];
        offset += byte_index + consumed_bytes;
    }
    push_val(PushGeneric::String(string))?;
    Ok(())
}

impl Error for DecodeError {
    fn description(&self) -> &str {
        match self.kind {
            DecodeErrorKind::UnescapedSlash => r#"Found unmatched '\'. Use "\\" to escape slashes"#,
            DecodeErrorKind::InvalidValue => r#"Escaped value is out of range of the decoder"#,
            DecodeErrorKind::HexNumberToShort => r#"Not enough characters after "\x" or "\u""#,
            DecodeErrorKind::InvalidHexDigit => r#"Invalid hex digit after "\x" or "\u""#,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} when decoding {:?} [index={}]",
            self.index, self, self.mat
        )
    }
}

#[cfg(test)]
mod error_tests {
    use crate::{decode::PushGeneric, DecodeError, DecodeErrorKind};

    use super::decode_generic;

    fn do_error_test(string: &str, err_index: usize, err_kind: DecodeErrorKind) {
        let mut out: Vec<u8> = Vec::new();
        let f = |val: PushGeneric| -> Result<(), DecodeError> {
            match val {
                PushGeneric::Value { val, start: _ } => {
                    out.push(val as u8);
                    Ok(())
                }
                PushGeneric::String(s) => {
                    out.extend_from_slice(s.as_bytes());
                    Ok(())
                }
            }
        };

        let result = decode_generic(f, string);

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err_index, err.index);
        assert_eq!(err_kind, err.kind);
    }

    #[test]
    fn test_error_unescaped_backslash() {
        do_error_test(r"foo\bar", 3, DecodeErrorKind::UnescapedSlash)
    }

    #[test]
    fn test_error_unescaped_backslash_2() {
        do_error_test(r"foo\n\bar", 5, DecodeErrorKind::UnescapedSlash)
    }

    #[test]
    fn test_error_unescaped_backslash_end() {
        do_error_test(r"foo\", 3, DecodeErrorKind::UnescapedSlash)
    }

    #[test]
    fn test_error_unescaped_backslash_end_2() {
        do_error_test(r"foo\nbar\", 8, DecodeErrorKind::UnescapedSlash);
    }

    #[test]
    fn test_error_escape_no_digits() {
        do_error_test(r"foo\nbar\x", 8, DecodeErrorKind::HexNumberToShort);
    }

    #[test]
    fn test_error_short_x_escape() {
        do_error_test(r"foo\nbar\x1", 8, DecodeErrorKind::HexNumberToShort);
    }

    #[test]
    fn test_error_short_u_escape() {
        do_error_test(r"foo\nbar\u12345", 8, DecodeErrorKind::HexNumberToShort);
    }

    #[test]
    fn test_error_invalid_hex_char() {
        do_error_test(r"foo\nbar\xax", 8, DecodeErrorKind::InvalidHexDigit);
    }
}

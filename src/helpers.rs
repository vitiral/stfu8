/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use std::u16;
use std::u32;
use std::fmt::Write;

/// the only visible character we escape
pub(crate) const BSLASH: u8 = b'\\';
pub(crate) const BSLASH_U16: u16 = BSLASH as u16;

/// create `u8` from two bytes of hex
pub(crate) fn from_hex2(hex2: &[u8]) -> u8 {
    debug_assert_eq!(2, hex2.len());
    (from_hex(hex2[0]) << 4) + from_hex(hex2[1])
}

#[inline(always)]
/// Convert a hexidecimal character (`0-F`) into it's corresponding numerical value (0-15)
fn from_hex(b: u8) -> u8 {
    (b as char).to_digit(16).unwrap() as u8
}

const SURROGATE_OFFSET: i64 = 0x1_0000 - (0xD800 << 10) - 0xDC00;

/// Convert from UTF-16 to UTF-32.
///
/// Note: the input must be pre-validated UTF-16.
///
/// From: <http://unicode.org/faq/utf_bom.html/>
pub(crate) fn to_utf32(v: &[u16]) -> u32 {
    if v.len() == 1 {
        v[0] as u32
    } else if v.len() == 2 {
        let lead = v[0] as i64;
        let trail = v[1] as i64;
        ((lead << 10) + trail + SURROGATE_OFFSET) as u32
    } else {
        panic!("invalid len: {}", v.len());
    }
}

pub(crate) fn escape_u8(dst: &mut String, encoder: &super::Encoder, b: u8) {
    match b {
        b'\\' => dst.push_str(r"\\"),
        b'\t' => if encoder.encode_tab {
            dst.push_str("\\t");
        } else {
            dst.push(b as char);
        },
        b'\n' => if encoder.encode_line_feed {
            dst.push_str("\\n");
        } else {
            dst.push(b as char);
        },
        b'\r' => if encoder.encode_cariage {
            dst.push_str("\\r");
        } else {
            dst.push(b as char);
        },
        _ => write!(dst, r"\x{:0>2X}", b).unwrap(),
    }
}

pub(crate) fn escape_u16(dst: &mut String, c16: u16) {
    write!(dst, r"\u{:0>6X}", c16).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_conversions(s: &str, expect_suplimental: bool) {
        let mut got_suplimental = false;
        for c in s.chars() {
            let mut expected = [0_u16; 2];
            let mut c16 = [0_u16; 2];
            let expected = c.encode_utf16(&mut expected);
            let c16 = c.encode_utf16(&mut c16);
            if expected.len() == 2 {
                got_suplimental = true
            }
            assert_eq!(expected, c16);

            let c32 = to_utf32(&c16);
            assert_eq!(c as u32, c32);
        }

        assert_eq!(expect_suplimental, got_suplimental);
    }

    #[test]
    fn sanity_utf_conversion() {
        assert_conversions("foo bar", false);
        assert_conversions("foo bar", false);
        assert_conversions("¡ ¢ £ ¤ ¥ ¦ § ¨ © ª « ¬ ­", false);
        assert_conversions(" ʰ ʱ ʲ ʳ ʴ ʵ ʶ ʷ ʸ ʹ ʺ ʻ", false);
        assert_conversions("܀ ܁ ܂ ܃ ܄ ܅ ܆ ܇ ܈ ܉ ܊ ܋ ܌ ܍ ܏", false);
        assert_conversions("Ꭰ Ꭱ Ꭲ Ꭳ Ꭴ Ꭵ Ꭶ Ꭷ Ꭸ Ꭹ", false);
        assert_conversions("ἀ ἁ ἂ ἃ ἄ ἅ ἆ ἇ Ἀ Ἁ", false);
        assert_conversions(
            "                          ​ ‌ ‍ ‎ ‏ ‐ ",
            false,
        );
        assert_conversions("‑ ‒ – — ― ‖ ‗ ‘ ’ ‚ ‛ “", false);
        assert_conversions("    ⃐ ⃑ ⃒ ⃓ ⃔ ⃕ ⃖ ⃗ ⃘ ⃙ ⃚ ⃛ ⃜ ⃝ ⃞ ⃟ ⃠ ⃡ ⃢ ⃣ ⃤ ⃥ ⃦ ⃧ ⃨ ⃩ ⃪ ", false);
        assert_conversions(
            "⟰ ⟱ ⟲ ⟳ ⟴ ⟵ ⟶ ⟷ ⟸ ⟹ ⟺ ⟻ ⟼ ⟽ ⟾ ⟿",
            false,
        );

        // suplimentary codes:
        assert_conversions(
            "𠜎 𠜎 𠜱 𠜱 𠝹 𠝹 𠱓 𠱓 𠱸 𠱸 𠲖 𠲖 𠳏 𠳏 𠳕 𠳕 𠴕 𠴕 𠵼 𠵼 𠵿 𠵿 𠸎 𠸎 𠸏 𠸏",
            true,
        );
    }
}

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

use std::u16;
use std::char;

use helpers;

/*
Section: UTF-8 validation
*/

const LEAD_MIN: u16 = 0xD800;
const LEAD_MAX: u16 = 0xDBFF;
const TRAIL_MIN: u16 = 0xDC00;
const TRAIL_MAX: u16 = 0xDFFF;

/// Encode u16 (i.e. almost UTF-16) into STFU-8.
pub(crate) fn encode(encoder: &super::Encoder, v: &[u16]) -> String {
    let mut out = String::with_capacity(v.len() * 2);

    let mut iter = v.iter();
    let mut c16 = match iter.next() {
        Some(c) => *c,
        None => return out,
    };

    loop {
        match c16 {
            // non-printable ascii
            0x00..=0x1F | helpers::BSLASH_U16 => helpers::escape_u8(&mut out, encoder, c16 as u8),
            // leading surrogates
            LEAD_MIN..=LEAD_MAX => {
                let trail = match iter.next() {
                    Some(t) => *t,
                    None => {
                        // lead at end of u16 (no trail)
                        helpers::escape_u16(&mut out, c16);
                        break;
                    }
                };
                if !(TRAIL_MIN <= trail && trail <= TRAIL_MAX) {
                    // lead without a trail, just escape it and handle the char on the next
                    // loop
                    helpers::escape_u16(&mut out, c16);
                    c16 = trail;
                    continue;
                }
                // has both a lead and a trail -- is valid!
                let buf = [c16, trail];
                out.push(char::from_u32(helpers::to_utf32(&buf)).unwrap());
            }
            // unpaired trailing surrogates
            TRAIL_MIN..=TRAIL_MAX => {
                // trail without a lead
                helpers::escape_u16(&mut out, c16);
            }
            _ => {
                out.push(char::from_u32(helpers::to_utf32(&[c16])).unwrap());
            }
        }
        c16 = match iter.next() {
            Some(c) => *c,
            None => break,
        };
    }
    out
}

#[test]
fn sanity_encode() {
    fn enc(s: &str) -> String {
        let utf16: Vec<u16> = s.encode_utf16().collect();
        println!("utf16: {:?}", utf16);
        let out = encode(&super::Encoder::new(), &utf16);
        // validation, we may use from_utf8_unchecked in the future
        let _ = ::std::str::from_utf8(&out.as_bytes()).unwrap();
        out
    }
    fn assert_enc(s: &str) {
        assert_eq!(enc(s), s);
    }
    assert_enc("foo bar");
    assert_enc("foo bar");
    assert_enc("¡ ¢ £ ¤ ¥ ¦ § ¨ © ª « ¬ ­");
    assert_enc(" ʰ ʱ ʲ ʳ ʴ ʵ ʶ ʷ ʸ ʹ ʺ ʻ");
    assert_enc("܀ ܁ ܂ ܃ ܄ ܅ ܆ ܇ ܈ ܉ ܊ ܋ ܌ ܍ ܏");
    assert_enc("Ꭰ Ꭱ Ꭲ Ꭳ Ꭴ Ꭵ Ꭶ Ꭷ Ꭸ Ꭹ");
    assert_enc("ἀ ἁ ἂ ἃ ἄ ἅ ἆ ἇ Ἀ Ἁ");
    assert_enc("                          ​ ‌ ‍ ‎ ‏ ‐ ");
    assert_enc("‑ ‒ – — ― ‖ ‗ ‘ ’ ‚ ‛ “");
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

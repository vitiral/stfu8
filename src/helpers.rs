/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

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

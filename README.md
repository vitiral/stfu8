# STFU-8: Sorta Text Format in UTF-8

[![Build Status](https://travis-ci.org/vitiral/stfu8.svg?branch=master)](https://travis-ci.org/vitiral/stfu8)

STFU-8 is a hacky text encoding/decoding protocol for data that might be *not
quite* UTF-8 but is still mostly UTF-8. It is based on the syntax of the `repr`
created when you write (or print) binary text in rust, python, C or other
common programming languages.

Its primary purpose is to be able to allow a human to **visualize and edit**
"data" that is mostly (or fully) **visible** UTF-8 text. It encodes all non
visible or non UTF-8 compliant bytes as longform text (i.e. ESC becomes the
full string `r"\x1B"`).  It can also encode/decode ill-formed UTF-16.

Comparision to other formats:
- **UTF-8** (i.e. [`std::str`](https://doc.rust-lang.org/std/str/index.html)):
  UTF-8 is a standardized format for encoding human understandable text in any
  language on the planet. It is the reason the internet can be understood by
  almost anyone and should be the primary way that text is encoded. However,
  not everything that is "UTF-8 like" follows the standard exactly. For
  instance:
  - The linux command line defines ANSI escape codes to provide styles like
    color, bold, italic, etc. Even though almost everything printed to a
    terminal is UTF-8 text these "escape codes" might not be, and even
    if they are UTF-8, they are not visible characters.
  - Windows paths are not *necessarily* UTF-8 compliant as they can
    have [ill formed text][utf-16-ill-formed-text].
  - There might be other cases you can think of or want to create. In general,
    try _not_ to create more use cases if you don't have to.
- **rust's [OsStr](https://doc.rust-lang.org/std/ffi/struct.OsStr.html)**:
  OsStr is the "cross platform" type for handling system specific strings,
  mainly in file paths. Unlike STFU-8 it not (always) coercible into UTF-8
  and therefore cannot be serialized into JSON or other formats.
- **WTF-8** ([rust-wtf8](https://github.com/SimonSapin/rust-wtf8)): is great
  for interoperating with different UTF standards but cannot be used to
  transmit data over the internet. The
  [spec states](https://simonsapin.github.io/wtf-8/): "WTF-8 must not be used
  to represent text in a file format or for transmission over the Internet."
- **base64** ([`base64`](https://crates.io/crates/base64)): also encodes binary
  data as UTF-8. If your data is *actually binary* (i.e. not text) then use
  base64. However, if your data was formerly text (or mostly text) then
  encoding to base64 will make it completely un(human)readable.
- **Array[u8]**: obviously great if your data is *actually binary* (i.e. NOT
  TEXT) and you don't need to put it into a UTF-8 encoding.  However, an array
  of bytes (i.e. `[0x72, 0x65, 0x61, 0x64, 0x20, 0x69, 0x74]` is
  not human readable. Even if it were in pure ASCII the only ones who can read
  it efficiently are low-level programming Gods who have never figured out how
  to debug-print their ASCII.
- **STFU-8** (this crate): is "good" when you want to have only
  printable/hand-editable text (and your data is _mostly_ UTF-8) but the data
  might have a couple of binary/non-printable/ill-formed pieces. It is _very
  poor_ if your data is actually binary, requiring (on average) a mapping of
  4/1 for binary data.

[1]: https://simonsapin.github.io/wtf-8/

# Specification
In simple terms, encoded STFU-8 is itself *always valid unicode* which decodes
to binary (the binary is not necessarily UTF-8). It differs from unicode in
that single `\` items are illegal. The following patterns are legal:
- `\\`: decodes to the backward-slash (`\`) byte (`\x5c`)
- `\t`: decodes to the tab byte (`\x09`)
- `\n`: decodes to the newline byte (`\x0A`)
- `\r`: decodes to the linefeed byte (`\x0D`)
- `\xXX` where XX are exactly two case-insensitive hexidecimal digits: decodes
  to the `\xXX` byte, where `XX` is a hexidecimal number (example: `\x9F`,
  `\xaB` or `\x05`). This *never* gets resolved into a code point, the value
  is pushed directly into the decoder stream.
- `\uXXXXXX` where `XXXXXX` are exacty six case-insensitive hexidecimal digits,
  decodes to a 24bit number that *typically* represenents a unicode code point.
  If the value *is* a unicode code point it will always be decoded as such.
  Otherwise `stfu8` will attempt to store the value into the decoder (if the
  value is too large for the decoding type it will be an error).

`stfu8` provides 2 different categories of functions for encoding/decoding data
that are *not necessarily interoperable* (don't decode output created from `encode_u8`
with `decode_u16`).
- `encode_u8(&[u8]) -> String` and `decode_u8(&str) -> Vec<u8>`: encodes or
  decodes an array of `u8` values to/from STFU-8, primarily used for interfacing
  with binary/nonvisible data that is *almost* UTF-8.
- `encode_u16(&[u16]) -> String` and `decode_u16(&str) -> Vec<u16>`: encodes
  or decodes an array of `u16` values to/from STFU-8, primarily used for
  interfacing with legacy UTF-16 formats that may contain
  [ill formed text][utf-16-ill-formed-text] but also converts unprintable
  characters.

There are some general rules for encoding and decoding:
- If `\u...` cannot be resolved into a valid UTF code point it *must* fit into
  the decoder. For instance, trying to decode `"\u00DEED"` (which is an UTF-16
  Trail surrogage) using `decode_u8` will fail, but will succeed with
  `decode_u16`.
- No escaped values are *ever chained*. For example, `"\x01\x02"` will be
  `[0x01, 0x02]` **not** `[0x0102]` -- even if you use `decode_u16`.
- Values escaped with `\x...` are always copied verbatum into the decoder.
  I.e. `\xFF` is a valid UTF-32 code point, but if decoded with `decode_u8`
  it will be `0xFE` in the buffer, not two bytes of data as the UTF-8 character
  `'þ'`. Note that with `decode_u16` `0xFE` is a valid UTF-16 code point, so
  when re-encoded would be the `'þ'` character. Moral of the story: _don't mix
  inputs/outputs of the the `u8` and `u16` functions_.

> tab, newline, and line-feed characters are "visible", so encoding with them
> in "pretty form" is optional.

## UTF-16 Ill Formed Text
The problem is succinctly stated here:

> http://unicode.org/faq/utf_bom.html
>
> Q: How do I convert an unpaired UTF-16 surrogate to UTF-8?
>
> A different issue arises if an unpairedsurrogate is encountered when
> converting ill-formed UTF-16 data. By represented such an unpaired surrogate
> on its own as a 3-byte sequence, the resulting UTF-8 data stream would become
> ill-formed. While it faithfully reflects the nature of the input, Unicode
> conformance requires that encoding form conversion always results in valid
> data stream. Therefore a convertermust treat this as an error. [AF]

Also, from the [WTF-8 spec](https://simonsapin.github.io/wtf-8/#motivation)

> As a result, [unpaired] surrogates do occur in practice and need to be
> preserved. For example:
>
> In ECMAScript (a.k.a. JavaScript), a String value is defined as a sequence
> of 16-bit integers that usually represents UTF-16 text but may or may not
> be well-formed.  Windows applications normally use UTF-16, but the file
> system treats path and file names as an opaque sequence of WCHARs (16-bit
> code units).
>
> We say that strings in these systems are encoded in potentially
> ill-formed UTF-16 or WTF-16.

Basically: you can't (always) convert from UTF-16 to UTF-8 and it's a real
bummer. WTF-8, while _kindof_ an answer to this problem, doesn't allow me
to serialize UTF-16 into a UTF-8 format, send it to my webapp, edit it (as a
human), and send it back. That is what STFU-8 is for.

# LICENSE
The source code in this repository is Licensed under either of
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

The STFU-8 protocol/specification itself (including the name) is licensed under
CC0 Community commons and anyone should be able to reimplement or change it for
any purpose without need of attribution. However, using the same name for a
completely different protocol would probably confuse people so please don't do
it.


# STFU8: Sorta Text Format in UTF-8

STFU8 is a hacky text encoding/decoding protocol for data that might be *not
quite* UTF8 but is still mostly UTF8. It is based on the syntax of the `repr`
created when you write (or print) binary text in python, C or other common
programming languages.

Comparision to other formats:
- **UTF8**: UTF8 is a standardized format for encoding human understandable
  text in any language on the planet. It is the reason the internet can be
  understood by almost anyone and should be the primary way that text is
  encoded. However, not everything that is "UTF8-like" follows the standard
  exactly. For instance:
  - The linux command line defines ANSI escape codes to provide styles like
    color, bold, italic, etc. Even though almost everything printed to a
    terminal is UTF8 text, these "escape codes" are not.
  - Windows paths are not *necessarily* UTF8 compliant (but they are [WTF-8][1]
    compliant).
  - There might be other cases you can think of or want to create. In general,
    try _not_ to create more use cases if you don't have to.
- **STFU8**: is "good" when you want data is mostly UTF8 text that might have a
  couple of binary pieces. It is especailly good if you want to support hand
  writing text combined with binary sequences and then decoding back to full
  binary. It has _very poor_ if your data is actually binary, requiring (
  on average) _4 bytes_ per byte of binary.
- **base64**: `base64` is great for when your data is *actually binary* and
  you want to transmit it in UTF8 (i.e. in json). However, if your data
  was formerly text (or mostly text) but if it will be completely unreadable.
- **Array[u8]**: obviously great if your data is *actually binary* (i.e. NOT
  TEXT) and you don't need to put it into a UTF8-only encoding (i.e. JSON).
  However, an array of bytes (i.e. `['0x72', '0x65', '0x61', '0x64', '0x20',
  '0x69', '0x74']` is not human readable. Even if it were in pure ASCII the
  only ones who can read it efficiently are low-level programming Gods who have
  never figured out how to debug-print their ASCII.

[1]: https://simonsapin.github.io/wtf-8/

# Specification (lol)
In simple terms, encoded STFU8 is itself *always valid unicode* which decodes
to binary (the binary is not necessarily UTF8). It differs from unicode in
that single `\` items are illegal. The following patterns are legal:
- `\\`: decodes to the backward-slash (`\`) byte (`\x5c`)
- `\t`: decodes to the tab byte (`\x09`)
- `\n`: decodes to the newline byte (`\x0A`)
- `\r`: decodes to the linefeed byte (`\x0D`)
- `\xXX` where XX are exactly two case-insensitive hexidecimal digits: decodes
  to the `\xXX` byte, where `XX` is a hexidecimal number (example: `\x9F`,
  `\xaB` or `\x05`)

> tab, newline, and line-feed characters are technically valid UTF8, so encoding
> with them in "printable form" is optional.

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

The STFU8 protocol/specification(lol) itself (including the name) is licensed
under CC0 Community commons and anyone should be able to reimplement or change
it for any purpose without need of attribution. However, using the same name
for a completely different protocol would probably confuse people so please
don't do it.


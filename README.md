# email-format

"Internet Message Format" meticulously implemented for email construction
and validation, as defined in RFC 5322 and other RFCs.

[Documentation](https://docs.rs/email-format)

## Features

* **Parses** bytes into an Email structure (represented internally as a tree) and validates
  RFC 5322 "Internet Message Format" compliance.
  * Extensive RFC 5322 Parser/validator: If you generate an email using this crate, you are
    guaranteed that will be a valid RFC 5322 formatted email, or else you will get a ParseError.
    The only exception that I am currently aware of is that lines can be longer than 998
    characters (see issue #3).
* **Streams** an Email structure back into bytes.
* **Generates and modifies** Email structures using functions like `set_subject()`,
  `get_from()`, `clear_reply_to()`, `add_optional_field()`, etc.
* Integrates with [lettre](https://github.com/lettre/lettre)
  (enable optional feature `lettre`)
  and [mailstrom](https://github.com/mikedilger/mailstrom)
* Supports [chrono](https://github.com/chronotope/chrono) `DateTime`
  and [time](https://github.com/rust-lang/time) `Tm` for setting the `Date` field
  (enable optional feature `chrono` and/or `time`)

## Limitations

* Valid emails are 7-bit ASCII, and this crate requires all content to be 7-bit ASCII.
  The proper way to send richer content is to use a transfer encoding, and to set a
  `content-transfer-encoding` header. We don't yet offer any help in this regard, beyond
  the ability to add_optional_field(). You'll have to manage the encoding yourself.
  We plan to add convenience functions for this eventually (see issue #19)
* Obsolete email formats are not implemented in the parser. Therefore, it is not sufficient
  for parsing inbound emails if you need to recognize formats that were obsoleted in 2008.

## Plans (not yet implemented)

* Support for content-transfer-encodings (unicode via Quoted Printable or Base64 or otherwise)
* Support for email headers defined in other RFCs:
  * Support for RFC 6854 (updated From and Sender syntax)
  * Support for all headers registered at IANA (http://www.iana.org/assignments/message-headers/message-headers.xhtml)
* Support for MIME (RFC 2045, RFC 4021, RFC 2231, RFC 6352) using
  [mime_multipart](https://github.com/mikedilger/mime-multipart)
* Support for streaming of MIME parts from disk.

## History

This project was inspired by the earlier [email](https://github.com/niax/rust-email) crate,
but was reworked from scratch due to a number of significant differences in design,
implementation and interface.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

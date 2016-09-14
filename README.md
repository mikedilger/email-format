# email-format

Implementation of email data structure, builder and streaming thereof.

Features include (many of these are still TBD):

* Supports MIME multipart messages via streaming (large parts never need to occupy memory
  more than a buffer at a time)
* Extensive RFC 5322 Parser/validator for email composition
* Ergonomic function signatures
* Minimal copying of data
* Implements `email` crate's `SendableEmail`, and so it works with the `lettre` crate.

## Limitations

The parser is not sufficient for parsing incoming emails.  RFC 5322 specifically requires
such parsers to recognize obsolete syntax.  For generation of emails, obsolete syntax is
not necessary, and usage of such is obsoleted by the RFC.

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

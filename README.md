# email-format

"Internet Message Format" meticulously implemented for email construction
and validation, as defined in RFC 5322 and other RFCs.

## Features

* Extensive RFC 5322 Parser/validator

## Limitations

* Obsolete email formats are not implemented in the parser.  Therefore, it is not sufficient
  for parsing inbound emails if you need to recognize formats that were obsoleted in 2008.

## Plans (not yet implemented)

* Support for RFC 6854 (updated From and Sender syntax)
* Support for MIME (RFC 2045, RFC 4021, RFC 2231, RFC 6352)
* Support for streaming of MIME parts from disk.
* Support for all headers registered at IANA (http://www.iana.org/assignments/message-headers/message-headers.xhtml)
* More ergonomic function signatures
* Less copying of data
* Implementation of `email` crate's `SendableEmail`, and so it works with the `lettre` crate (will be inefficient due to the way SendableEmail is defined).

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

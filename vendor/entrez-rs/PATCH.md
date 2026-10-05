Source: https://crates.io/crates/entrez-rs/0.1.4 (MIT).

Local changes:

- Disable Reqwest default features and enable `rustls-tls` alongside `blocking`.
- Remove unused borrows of `String::push` results in the PubMed parser.
- Update quick-xml to 0.23.1 to fix future-incompatible macro syntax.
- Normalize trailing whitespace in the vendored files.

# TLS dependency

`entrez-rs` 0.1.4 enables Reqwest 0.10's default TLS features, pulling in
native-tls and OpenSSL. It does not expose a feature to disable them. The local
Cargo patch uses Rustls with WebPKI roots instead.

The vendored PubMed parser had unused borrows of `Vec::push` results.
Removing them preserves the generated abstract markup. quick-xml 0.20 and
0.22 trigger the future-incompatible `semicolon_in_expressions_from_macros`
lint; 0.23.1 builds without it.

Validation: `cargo check` and `cargo test --offline` pass without compiler
warnings. Regression tests cover ESearch XML deserialization and nested PubMed
abstract markup. The lockfile contains neither OpenSSL nor native-tls.

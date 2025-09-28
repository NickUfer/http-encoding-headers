# http_encoding_headers

[![Crates.io](https://img.shields.io/crates/v/http_encoding_headers.svg)](https://crates.io/crates/http_encoding_headers)
[![Documentation](https://docs.rs/http_encoding_headers/badge.svg)](https://docs.rs/http_encoding_headers)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust library for handling HTTP Accept-Encoding and Content-Encoding headers with support for common compression
algorithms and custom encodings.

## Features

- Parse and generate Accept-Encoding headers with quality values
- Parse and generate Content-Encoding headers
- Support for common encodings: gzip, deflate, br, zstd, and more
- Support for custom/unknown encodings via `Encoding::Custom`
- Integration with `http` and `headers` crates for encoding/decoding. Can optionally be turned off.

## License

MIT
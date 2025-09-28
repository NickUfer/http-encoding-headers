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

## Examples

### Basic Accept-Encoding Usage

```rust
use http_encoding_headers::{encode_header_value, decode_header_value, Encoding};

// Encode Accept-Encoding header
let encodings = vec![
    (Encoding::Gzip, 1.0),
    (Encoding::Deflate, 0.8), 
    (Encoding::Br, 0.6),
];

let header_value = encode_header_value(&encodings).unwrap();
// Result: "gzip, deflate;q=0.8, br;q=0.6"

// Decode Accept-Encoding header
let parsed = decode_header_value("gzip, deflate;q=0.8, br;q=0.6").unwrap();
// Result: [(Gzip, 1.0), (Deflate, 0.8), (Br, 0.6)]
```

### AcceptEncoding with Sorting and Preferred Encoding

```rust
    use http_encoding_headers::{AcceptEncoding, Encoding};

let encodings = vec![
    (Encoding::Gzip, 0.8),
    (Encoding::Br, 1.0),        // Highest preference  
    (Encoding::Deflate, 0.6),
];

let mut accept_encoding = AcceptEncoding::new(encodings).unwrap();

// Find preferred encoding (highest quality)
let preferred = accept_encoding.preferred(); // Some(Br)

// Find preferred encoding (highest quality) which is also in a list of allowed encodings
let preferred_allowed = accept_encoding.preferred_allowed(vec![Encoding::Br].iter());

// Sort in descending order (highest quality first)
accept_encoding.sort_descending();

// Sort in ascending order (lowest quality first)  
accept_encoding.sort_ascending();
```

### Content-Encoding Usage

```rust
use http_encoding_headers::{ContentEncoding, Encoding};
use headers::{Header, HeaderMapExt};
use http::HeaderMap;

// Create Content-Encoding header
let content_encoding = ContentEncoding::new(Encoding::Gzip);

// Use with HTTP HeaderMap
let mut headers = HeaderMap::new();
headers.typed_insert(content_encoding);

// Decode from header values
let header_values = vec![http::HeaderValue::from_str("gzip").unwrap()];
let decoded = ContentEncoding::decode(&mut header_values.iter()).unwrap();
```

### Running Examples

To see comprehensive examples in action:

```bash
# Basic usage examples
cargo run --example basic_usage

# AcceptEncoding specific examples  
cargo run --example accept_encoding_preferred

# Encode/decode focused examples
cargo run --example encode_decode

# Axum integration example
cargo run --manifest-path examples/axum_integration/Cargo.toml
```

#### Axum Integration Example

The [axum_integration.rs](examples/axum_integration) example demonstrates real-world usage with the Axum web framework:

```rust
use axum_extra::extract::TypedHeader;
use http_encoding_headers::AcceptEncoding;

// AcceptEncoding can be injected directly as a TypedHeader parameter
async fn handler(accept_encoding: Option<TypedHeader<AcceptEncoding>>) -> impl IntoResponse {
    match accept_encoding {
        Some(TypedHeader(accept_encoding)) => {
            let preferred = accept_encoding.preferred();
            // Use preferred encoding for content negotiation
            format!("Preferred encoding: {:?}", preferred)
        },
        None => "No Accept-Encoding header".to_string(),
    }
}
```

The axum example includes:
- TypedHeader extraction for AcceptEncoding
- Content negotiation algorithms
- Setting Content-Encoding response headers
- Multiple handler patterns and JSON responses

## License

MIT
//! Focused examples for encoding and decoding HTTP headers
//!
//! This demonstrates:
//! - encode_header_value() and decode_header_value() functions
//! - Content-Encoding header operations
//! - Error handling for invalid header values

use http_encoding_headers::{
    AcceptEncodingDecodeError, AcceptEncodingEncodeError, ContentEncoding, Encoding,
    decode_header_value, encode_header_value,
};

#[cfg(feature = "http_crates")]
use headers::Header;
#[cfg(feature = "http_crates")]
use http::HeaderValue;

fn main() {
    println!("=== Encoding and Decoding Examples ===\n");

    // Accept-Encoding examples
    accept_encoding_encode_decode_examples();

    // Content-Encoding examples
    content_encoding_examples();

    // Error handling examples
    error_handling_examples();
}

fn accept_encoding_encode_decode_examples() {
    println!("1. Accept-Encoding Encode/Decode");
    println!("=================================");

    // Example 1: Basic encoding
    println!("\n1a. Basic encoding:");
    let encodings = vec![
        (Encoding::Gzip, 1.0),
        (Encoding::Deflate, 0.8),
        (Encoding::Br, 0.6),
    ];

    match encode_header_value(&encodings) {
        Ok(encoded) => println!("   Encoded: {}", encoded),
        Err(e) => println!("   Error: {}", e),
    }

    // Example 2: Encoding with quality value formatting
    println!("\n1b. Quality value formatting:");
    let encodings_with_various_qualities = vec![
        (Encoding::Gzip, 1.0),       // q=1.0 omitted
        (Encoding::Deflate, 0.500),  // trailing zeros trimmed
        (Encoding::Br, 0.123),       // precise value
        (Encoding::Identity, 0.100), // trailing zeros trimmed
    ];

    match encode_header_value(&encodings_with_various_qualities) {
        Ok(encoded) => {
            println!("   Input qualities: [1.0, 0.500, 0.123, 0.100]");
            println!("   Encoded: {}", encoded);
            println!("   (Note: q=1.0 omitted, trailing zeros trimmed)");
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 3: Basic decoding
    println!("\n1c. Basic decoding:");
    let header_values = vec![
        "gzip",
        "gzip, deflate",
        "gzip, deflate;q=0.8, br;q=0.6",
        "br;q=1.0, gzip;q=0.8, deflate;q=0.5, *;q=0.1",
    ];

    for header in header_values {
        println!("   Decoding: \"{}\"", header);
        match decode_header_value(header) {
            Ok(parsed) => {
                println!("   Result:");
                for (encoding, quality) in parsed {
                    println!("     {} (q={})", encoding, quality);
                }
            }
            Err(e) => println!("   Error: {}", e),
        }
        println!();
    }

    // Example 4: Round-trip encoding/decoding
    println!("1d. Round-trip encoding/decoding:");
    let original = vec![
        (Encoding::Gzip, 1.0),
        (Encoding::Deflate, 0.8),
        (Encoding::Custom("custom-encoding".to_string()), 0.5),
    ];

    println!("   Original: {:?}", original);

    if let Ok(encoded) = encode_header_value(&original) {
        println!("   Encoded: {}", encoded);

        if let Ok(decoded) = decode_header_value(&encoded) {
            println!("   Decoded: {:?}", decoded);

            // Verify they match
            let matches = original.len() == decoded.len()
                && original
                    .iter()
                    .zip(decoded.iter())
                    .all(|((enc1, q1), (enc2, q2))| enc1 == enc2 && (q1 - q2).abs() < f32::EPSILON);
            println!("   Round-trip successful: {}", matches);
        }
    }
}

#[cfg(feature = "http_crates")]
fn content_encoding_examples() {
    println!("\n\n2. Content-Encoding Examples");
    println!("============================");

    // Example 1: Creating Content-Encoding
    println!("\n2a. Creating Content-Encoding headers:");
    let encodings_to_test = vec![
        Encoding::Gzip,
        Encoding::Deflate,
        Encoding::Br,
        Encoding::Zstd,
        Encoding::Custom("lz4".to_string()),
    ];

    for encoding in encodings_to_test {
        let content_encoding = ContentEncoding::new(encoding);
        println!("   Created: {:?}", content_encoding);

        // Encode to header value
        let mut values = Vec::new();
        content_encoding.encode(&mut values);

        if let Some(header_value) = values.first() {
            if let Ok(as_str) = header_value.to_str() {
                println!("     Header value: {}", as_str);
            }
        }
    }

    // Example 2: Decoding Content-Encoding headers
    println!("\n2b. Decoding Content-Encoding headers:");
    let test_headers = vec!["gzip", "deflate", "br", "zstd", "custom-encoding"];

    for header_str in test_headers {
        println!("   Decoding: \"{}\"", header_str);

        let header_values = vec![HeaderValue::from_str(header_str).unwrap()];
        match ContentEncoding::decode(&mut header_values.iter()) {
            Ok(decoded) => {
                println!("     Success: {:?}", decoded);
                println!("     Encoding: {:?}", decoded.encoding());
            }
            Err(e) => println!("     Error: {:?}", e),
        }
    }
}

#[cfg(not(feature = "http_crates"))]
fn content_encoding_examples() {
    println!("\n\n2. Content-Encoding Examples");
    println!("============================");
    println!("   (Skipped - http_crates feature not enabled)");
}

fn error_handling_examples() {
    println!("\n\n3. Error Handling Examples");
    println!("==========================");

    // Example 1: Encoding errors
    println!("\n3a. Accept-Encoding encoding errors:");

    // Empty encodings list
    match encode_header_value(&[]) {
        Ok(_) => println!("   Unexpected success"),
        Err(AcceptEncodingEncodeError::EmptyEncodings) => {
            println!("   ✓ Correctly caught empty encodings error");
        }
        Err(_) => println!("   Unexpected error type"),
    }

    // Example 2: Decoding errors
    println!("\n3b. Accept-Encoding decoding errors:");

    let invalid_headers = vec![
        "",               // Empty string
        " , gzip",        // Empty encoding name
        ";q=1.0",         // Missing encoding name
        "gzip;q=invalid", // Invalid quality value
        "gzip;foo=bar",   // Unknown directive
    ];

    for invalid_header in invalid_headers {
        println!("   Testing: \"{}\"", invalid_header);
        match decode_header_value(invalid_header) {
            Ok(_) => println!("     Unexpected success"),
            Err(AcceptEncodingDecodeError::EmptyEncodingName) => {
                println!("     ✓ Empty encoding name error");
            }
            Err(AcceptEncodingDecodeError::EmptyEncodingWeightTuple) => {
                println!("     ✓ Empty encoding weight tuple error");
            }
            Err(AcceptEncodingDecodeError::InvalidQualityValue(val)) => {
                println!("     ✓ Invalid quality value error: {}", val);
            }
            Err(AcceptEncodingDecodeError::UnexpectedDirective(directive)) => {
                println!("     ✓ Unexpected directive error: {}", directive);
            }
            Err(_) => println!("     ✓ Other decode error"),
        }
    }

    #[cfg(feature = "http_crates")]
    {
        // Example 3: Content-Encoding conflicting values
        println!("\n3c. Content-Encoding conflicting values:");
        let conflicting = vec![
            HeaderValue::from_str("gzip").unwrap(),
            HeaderValue::from_str("deflate").unwrap(),
        ];

        match ContentEncoding::decode(&mut conflicting.iter()) {
            Ok(_) => println!("   Unexpected success"),
            Err(_) => println!("   ✓ Correctly rejected conflicting Content-Encoding values"),
        }
    }
}

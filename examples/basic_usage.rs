//! Basic usage examples for http_encoding_headers
//!
//! This example demonstrates:
//! - Encoding and decoding Accept-Encoding headers
//! - Encoding and decoding Content-Encoding headers
//! - Using AcceptEncoding methods like preferred() and sorting

use http_encoding_headers::{
    AcceptEncoding, ContentEncoding, Encoding, decode_header_value, encode_header_value,
};

#[cfg(feature = "http_crates")]
use headers::{Header, HeaderMapExt};
#[cfg(feature = "http_crates")]
use http::{HeaderMap, HeaderValue};

fn main() {
    println!("=== HTTP Encoding Headers Examples ===\n");

    // Example 1: Accept-Encoding header encoding and decoding
    accept_encoding_examples();

    // Example 2: Content-Encoding header encoding and decoding
    content_encoding_examples();

    // Example 3: AcceptEncoding advanced usage
    accept_encoding_advanced_examples();
}

/// Examples for Accept-Encoding header encode/decode functionality
fn accept_encoding_examples() {
    println!("1. Accept-Encoding Header Examples");
    println!("==================================");

    // Example 1a: Encoding Accept-Encoding header values
    println!("\n1a. Encoding Accept-Encoding header values:");
    let encodings = vec![
        (Encoding::Gzip, 1.0),
        (Encoding::Deflate, 0.8),
        (Encoding::Br, 0.6),
        (Encoding::Identity, 0.1),
    ];

    match encode_header_value(&encodings) {
        Ok(header_value) => {
            println!("   Input: {:?}", encodings);
            println!("   Encoded header: {}", header_value);
        }
        Err(e) => println!("   Error encoding: {}", e),
    }

    // Example 1b: Decoding Accept-Encoding header values
    println!("\n1b. Decoding Accept-Encoding header values:");
    let header_string = "gzip, deflate;q=0.8, br;q=0.6, identity;q=0.1";
    println!("   Input header: {}", header_string);

    match decode_header_value(header_string) {
        Ok(parsed) => {
            println!("   Decoded encodings:");
            for (encoding, quality) in parsed {
                println!("     {} (q={})", encoding, quality);
            }
        }
        Err(e) => println!("   Error decoding: {}", e),
    }

    // Example 1c: Working with complex Accept-Encoding values
    println!("\n1c. Complex Accept-Encoding example:");
    let complex_header = "gzip;q=1.0, deflate;q=0.5, br;q=0.25, *;q=0.1";
    println!("   Input: {}", complex_header);

    if let Ok(parsed) = decode_header_value(complex_header) {
        // Create AcceptEncoding instance
        if let Ok(accept_encoding) = AcceptEncoding::new(parsed) {
            println!(
                "   Created AcceptEncoding with {} encodings",
                accept_encoding.items().len()
            );

            // Re-encode it
            if let Ok(re_encoded) = encode_header_value(accept_encoding.items()) {
                println!("   Re-encoded: {}", re_encoded);
            }
        }
    }
}

/// Examples for Content-Encoding header encode/decode functionality  
#[cfg(feature = "http_crates")]
fn content_encoding_examples() {
    println!("\n\n2. Content-Encoding Header Examples");
    println!("===================================");

    // Example 2a: Creating and encoding Content-Encoding headers
    println!("\n2a. Creating and encoding Content-Encoding:");
    let content_encoding = ContentEncoding::new(Encoding::Gzip);
    println!("   Created ContentEncoding: {:?}", content_encoding);

    // Use with HeaderMap
    let mut headers = HeaderMap::new();
    headers.typed_insert(content_encoding);

    if let Some(header_value) = headers.get(http::header::CONTENT_ENCODING) {
        println!(
            "   Header value: {}",
            header_value.to_str().unwrap_or("invalid")
        );
    }

    // Example 2b: Decoding Content-Encoding headers
    println!("\n2b. Decoding Content-Encoding headers:");
    let test_values = vec!["gzip", "deflate", "br", "zstd"];

    for encoding_str in test_values {
        println!("   Testing: {}", encoding_str);
        let header_values = vec![HeaderValue::from_str(encoding_str).unwrap()];

        match ContentEncoding::decode(&mut header_values.iter()) {
            Ok(decoded) => println!("     Decoded: {:?}", decoded),
            Err(e) => println!("     Error: {:?}", e),
        }
    }

    // Example 2c: Multiple identical values (valid)
    println!("\n2c. Multiple identical Content-Encoding values:");
    let identical_values = vec![
        HeaderValue::from_str("gzip").unwrap(),
        HeaderValue::from_str("gzip").unwrap(),
    ];

    match ContentEncoding::decode(&mut identical_values.iter()) {
        Ok(decoded) => println!("   Multiple identical values decoded: {:?}", decoded),
        Err(e) => println!("   Error: {:?}", e),
    }

    // Example 2d: Conflicting values (should error)
    println!("\n2d. Conflicting Content-Encoding values (should error):");
    let conflicting_values = vec![
        HeaderValue::from_str("gzip").unwrap(),
        HeaderValue::from_str("deflate").unwrap(),
    ];

    match ContentEncoding::decode(&mut conflicting_values.iter()) {
        Ok(decoded) => println!("   Unexpectedly decoded: {:?}", decoded),
        Err(_) => println!("   Correctly rejected conflicting values"),
    }
}

#[cfg(not(feature = "http_crates"))]
fn content_encoding_examples() {
    println!("\n\n2. Content-Encoding Header Examples");
    println!("===================================");
    println!("   (Skipped - http_crates feature not enabled)");
}

/// Examples for AcceptEncoding advanced functionality
fn accept_encoding_advanced_examples() {
    println!("\n\n3. AcceptEncoding Advanced Usage");
    println!("===============================");

    // Create an AcceptEncoding instance with various encodings
    let encodings = vec![
        (Encoding::Gzip, 0.9),
        (Encoding::Deflate, 0.8),
        (Encoding::Br, 1.0),       // Highest quality
        (Encoding::Identity, 0.1), // Lowest quality
        (Encoding::Zstd, 0.7),
    ];

    let mut accept_encoding = AcceptEncoding::new(encodings).unwrap();
    println!("\n3a. Original AcceptEncoding:");
    println!("   Encodings: {:?}", accept_encoding.items());

    // Example 3a: Finding preferred encoding
    println!("\n3b. Finding preferred encoding:");
    if let Some(preferred) = accept_encoding.preferred() {
        println!("   Preferred encoding: {}", preferred);
        println!("   (Highest quality value from unsorted list)");
    }

    // Example 3b: Sorting in descending order (highest quality first)
    println!("\n3c. Sorting in descending order (highest quality first):");
    accept_encoding.sort_descending();
    println!("   After sort_descending():");
    for (encoding, quality) in accept_encoding.items() {
        println!("     {} (q={})", encoding, quality);
    }

    // Now preferred should be first item
    if let Some(preferred) = accept_encoding.preferred() {
        println!("   Preferred encoding after sorting: {}", preferred);
        println!("   (First item in descending sorted list)");
    }

    // Example 3c: Sorting in ascending order (lowest quality first)
    println!("\n3d. Sorting in ascending order (lowest quality first):");
    accept_encoding.sort_ascending();
    println!("   After sort_ascending():");
    for (encoding, quality) in accept_encoding.items() {
        println!("     {} (q={})", encoding, quality);
    }

    // Now preferred should be last item
    if let Some(preferred) = accept_encoding.preferred() {
        println!("   Preferred encoding after ascending sort: {}", preferred);
        println!("   (Last item in ascending sorted list)");
    }

    // Example 3d: Demonstrating in-place sorting behavior
    println!("\n3e. Demonstrating in-place sorting chain:");
    let encodings2 = vec![
        (Encoding::Gzip, 0.5),
        (Encoding::Deflate, 0.9),
        (Encoding::Br, 0.3),
    ];

    let mut accept_encoding2 = AcceptEncoding::new(encodings2).unwrap();
    println!("   Original: {:?}", accept_encoding2.items());

    // Chain sorting operations (returns &mut Self for chaining)
    accept_encoding2.sort_descending();
    println!("   Descending: {:?}", accept_encoding2.items());

    // Sort back to ascending
    accept_encoding2.sort_ascending();
    println!("   Ascending: {:?}", accept_encoding2.items());

    println!("\n3f. Practical example - Content negotiation:");
    practical_content_negotiation_example();
}

/// A practical example showing how to use these functions for content negotiation
fn practical_content_negotiation_example() {
    // Simulate a client's Accept-Encoding header
    let client_header = "br;q=1.0, gzip;q=0.8, deflate;q=0.6, *;q=0.1";
    println!("   Client Accept-Encoding: {}", client_header);

    // Server supported encodings (in order of preference)
    let server_supported = vec![Encoding::Gzip, Encoding::Deflate, Encoding::Identity];
    println!("   Server supported: {:?}", server_supported);

    // Parse client preferences
    if let Ok(client_encodings) = decode_header_value(client_header) {
        if let Ok(mut accept_encoding) = AcceptEncoding::new(client_encodings) {
            // Sort by client preference (highest quality first)
            accept_encoding.sort_descending();

            // Find the best match
            let mut selected_encoding = None;
            for (encoding, quality) in accept_encoding.items() {
                if server_supported.contains(encoding) && *quality > 0.0 {
                    selected_encoding = Some(encoding);
                    break;
                }
            }

            match selected_encoding {
                Some(encoding) => {
                    println!("   Selected encoding: {}", encoding);
                    println!("   Server should use Content-Encoding: {}", encoding);
                }
                None => println!("   No acceptable encoding found"),
            }
        }
    }
}

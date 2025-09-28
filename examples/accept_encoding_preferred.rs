//! Simple example demonstrating AcceptEncoding preferred() and preferred_allowed() methods with in-place sorting
//!
//! This example shows:
//! - How to find the preferred encoding from an AcceptEncoding instance using preferred()
//! - How to find the preferred encoding with server-side filtering using preferred_allowed()
//! - How sorting affects the preferred encoding selection
//! - In-place sorting with sort_descending() and sort_ascending()

use http_encoding_headers::{AcceptEncoding, Encoding};

fn main() {
    println!("=== AcceptEncoding Preferred and Sorting Examples ===\n");

    // Create an AcceptEncoding with different quality values
    let encodings = vec![
        (Encoding::Gzip, 0.7),
        (Encoding::Deflate, 0.9), // This should be preferred (highest quality)
        (Encoding::Br, 1.0),      // Highest quality, but might not be allowed by server
        (Encoding::Identity, 0.1),
        (Encoding::Zstd, 0.8), // High quality compression
    ];

    let mut accept_encoding = AcceptEncoding::new(encodings).unwrap();

    println!("Original encodings:");
    print_encodings(&accept_encoding);

    // Find preferred encoding from unsorted list
    println!("\n1. Finding preferred encoding (unsorted) - preferred():");
    if let Some(preferred) = accept_encoding.preferred() {
        println!(
            "   Preferred: {} (finds highest quality from unsorted list)",
            preferred
        );
    }

    // Example server capabilities - let's say server only supports some encodings
    let server_supported = vec![Encoding::Gzip, Encoding::Deflate, Encoding::Identity];

    println!("\n2. Server-side filtering with preferred_allowed():");
    println!("   Server supports: {:?}", server_supported);
    if let Some(preferred_allowed) = accept_encoding.preferred_allowed(server_supported.iter()) {
        println!(
            "   Preferred (server filtered): {} (highest quality that server supports)",
            preferred_allowed
        );
    }

    // Compare different server capabilities
    let limited_server = vec![Encoding::Identity]; // Very limited server
    println!("\n   Limited server supports only: {:?}", limited_server);
    if let Some(preferred_limited) = accept_encoding.preferred_allowed(limited_server.iter()) {
        println!(
            "   Preferred (limited server): {} (only available option)",
            preferred_limited
        );
    } else {
        println!("   No acceptable encoding found for limited server");
    }

    let advanced_server = vec![Encoding::Br, Encoding::Zstd, Encoding::Gzip]; // Advanced server
    println!("\n   Advanced server supports: {:?}", advanced_server);
    if let Some(preferred_advanced) = accept_encoding.preferred_allowed(advanced_server.iter()) {
        println!(
            "   Preferred (advanced server): {} (best match from advanced capabilities)",
            preferred_advanced
        );
    }

    // Sort descending (highest quality first) - modifies in place
    println!("\n3. After sorting descending (in-place):");
    accept_encoding.sort_descending();
    print_encodings(&accept_encoding);

    if let Some(preferred) = accept_encoding.preferred() {
        println!(
            "   Preferred: {} (first item when sorted descending)",
            preferred
        );
    }

    // Test preferred_allowed with sorted list
    println!("\n   Server-side filtering after sorting descending:");
    if let Some(preferred_allowed) = accept_encoding.preferred_allowed(server_supported.iter()) {
        println!(
            "   Preferred (server filtered): {} (efficiently finds first match in sorted list)",
            preferred_allowed
        );
    }

    // Sort ascending (lowest quality first) - modifies in place
    println!("\n4. After sorting ascending (in-place):");
    accept_encoding.sort_ascending();
    print_encodings(&accept_encoding);

    if let Some(preferred) = accept_encoding.preferred() {
        println!(
            "   Preferred: {} (last item when sorted ascending)",
            preferred
        );
    }

    // Test preferred_allowed with ascending sorted list
    println!("\n   Server-side filtering after sorting ascending:");
    if let Some(preferred_allowed) = accept_encoding.preferred_allowed(server_supported.iter()) {
        println!(
            "   Preferred (server filtered): {} (efficiently finds best match from end)",
            preferred_allowed
        );
    }

    // Demonstrate chaining - sort methods return &mut Self for chaining
    println!("\n5. Method chaining example:");
    let encodings2 = vec![
        (Encoding::Gzip, 0.3),
        (Encoding::Deflate, 0.8),
        (Encoding::Br, 0.6),
    ];

    let mut accept_encoding2 = AcceptEncoding::new(encodings2).unwrap();

    // Chain operations
    let preferred_after_desc_sort = accept_encoding2.sort_descending().preferred().cloned(); // Clone the encoding to avoid borrowing issues

    println!(
        "   After chaining sort_descending().preferred(): {:?}",
        preferred_after_desc_sort
    );

    // Sort back and get preferred
    let preferred_after_asc_sort = accept_encoding2.sort_ascending().preferred().cloned();

    println!(
        "   After chaining sort_ascending().preferred(): {:?}",
        preferred_after_asc_sort
    );

    // Demonstrate with equal quality values
    println!("\n6. Equal quality values example:");
    let equal_encodings = vec![
        (Encoding::Gzip, 0.8),
        (Encoding::Deflate, 0.8), // Same quality
        (Encoding::Br, 0.8),      // Same quality
    ];

    let mut equal_accept = AcceptEncoding::new(equal_encodings).unwrap();
    println!("   Original (all equal quality):");
    print_encodings(&equal_accept);

    if let Some(preferred) = equal_accept.preferred() {
        println!(
            "   Preferred from equal qualities: {} (first found with max quality)",
            preferred
        );
    }

    // Test server filtering with equal qualities
    let partial_server = vec![Encoding::Deflate, Encoding::Br];
    if let Some(preferred_filtered) = equal_accept.preferred_allowed(partial_server.iter()) {
        println!(
            "   Preferred (server filtered from equal qualities): {} (first allowed match)",
            preferred_filtered
        );
    }

    equal_accept.sort_descending();
    println!("   After sort_descending (stable sort preserves original order for equal elements):");
    print_encodings(&equal_accept);

    // Demonstrate practical content negotiation scenario
    println!("\n7. Practical content negotiation scenario:");
    practical_negotiation_example();
}

fn print_encodings(accept_encoding: &AcceptEncoding) {
    for (encoding, quality) in accept_encoding.items() {
        println!("     {} (q={})", encoding, quality);
    }
}

/// Demonstrates practical content negotiation comparing both methods
fn practical_negotiation_example() {
    println!(
        "   Scenario: Client sends 'Accept-Encoding: br;q=1.0, zstd;q=0.9, gzip;q=0.7, deflate;q=0.5'"
    );

    let client_preferences = vec![
        (Encoding::Br, 1.0),      // Client's top choice
        (Encoding::Zstd, 0.9),    // Second choice
        (Encoding::Gzip, 0.7),    // Third choice
        (Encoding::Deflate, 0.5), // Fallback
    ];

    let accept_encoding = AcceptEncoding::new(client_preferences).unwrap();

    // Method 1: preferred() - ignores server capabilities
    if let Some(client_preferred) = accept_encoding.preferred() {
        println!(
            "   preferred(): {} (client's top choice, ignoring server)",
            client_preferred
        );
    }

    // Different server scenarios
    let scenarios = vec![
        (
            "Basic server",
            vec![Encoding::Gzip, Encoding::Deflate, Encoding::Identity],
        ),
        (
            "Advanced server",
            vec![Encoding::Br, Encoding::Zstd, Encoding::Gzip],
        ),
        ("Legacy server", vec![Encoding::Deflate, Encoding::Identity]),
        (
            "Modern server",
            vec![
                Encoding::Br,
                Encoding::Zstd,
                Encoding::Gzip,
                Encoding::Deflate,
            ],
        ),
    ];

    for (server_name, server_caps) in scenarios {
        print!("   {}: supports {:?}", server_name, server_caps);

        if let Some(negotiated) = accept_encoding.preferred_allowed(server_caps.iter()) {
            println!(" → selected: {}", negotiated);
        } else {
            println!(" → no acceptable encoding found");
        }
    }

    println!("\n   Key difference:");
    println!("   - preferred(): Returns client's top choice regardless of server capabilities");
    println!("   - preferred_allowed(): Returns best mutually supported encoding");
    println!("   - Both methods honor the current sorting state for efficiency");
}

//! Axum integration example showing how to use AcceptEncoding with TypedHeader
//!
//! This example demonstrates:
//! - Using AcceptEncoding as a TypedHeader in axum route handlers
//! - Content negotiation based on client preferences
//! - Responding with appropriate Content-Encoding headers
//! - Error handling for missing or invalid Accept-Encoding headers

use axum::{extract::{Query, State}, http, http::StatusCode, response::{IntoResponse, Response}, routing::get, Json, Router};
use axum_extra::extract::TypedHeader;
use http_encoding_headers::{AcceptEncoding, ContentEncoding, Encoding};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

/// Example JSON response data
#[derive(Serialize)]
struct ApiResponse {
    message: String,
    encoding_used: String,
    client_preferences: Vec<EncodingPreference>,
    server_capabilities: Vec<String>,
}

#[derive(Serialize)]
struct EncodingPreference {
    encoding: String,
    quality: f32,
}

/// Query parameters for controlling server behavior
#[derive(Deserialize)]
struct ServerConfig {
    /// Comma-separated list of encodings the server supports
    #[serde(default = "default_server_encodings")]
    supported: String,
}

fn default_server_encodings() -> String {
    "gzip,deflate,br".to_string()
}

/// Application state containing server configuration
#[derive(Clone)]
struct AppState {
    supported_encodings: Vec<Encoding>,
}

#[tokio::main]
async fn main() {
    println!("Starting Axum server with AcceptEncoding integration...");

    // Default server capabilities
    let state = AppState {
        supported_encodings: vec![
            Encoding::Gzip,
            Encoding::Deflate,
            Encoding::Br,
            Encoding::Identity,
        ],
    };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/data", get(api_handler))
        .route("/negotiate", get(negotiate_handler))
        .with_state(state);

    println!("Server running on http://localhost:3000");
    println!("\nAvailable endpoints:");
    println!("  GET /                 - Simple AcceptEncoding extraction example");
    println!("  GET /api/data         - Content negotiation with JSON response");
    println!("  GET /negotiate        - Advanced negotiation with server config");
    println!("\nExample requests:");
    println!("  curl -H 'Accept-Encoding: gzip, deflate;q=0.8, br;q=0.9' http://localhost:3000/");
    println!("  curl -H 'Accept-Encoding: br;q=1.0, gzip;q=0.5' http://localhost:3000/api/data");
    println!("  curl 'http://localhost:3000/negotiate?supported=gzip,zstd' -H 'Accept-Encoding: zstd, gzip;q=0.8'");

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Simple handler demonstrating AcceptEncoding extraction
async fn root_handler(
    // TypedHeader automatically parses the Accept-Encoding header into AcceptEncoding
    accept_encoding: Option<TypedHeader<AcceptEncoding>>,
) -> impl IntoResponse {
    match accept_encoding {
        Some(TypedHeader(accept_encoding)) => {
            let mut response_lines = vec![
                "Accept-Encoding header found!".to_string(),
                "".to_string(),
                "Client preferences:".to_string(),
            ];

            // Show all encodings with their quality values
            for (encoding, quality) in accept_encoding.items() {
                response_lines.push(format!("  {} (q={})", encoding, quality));
            }

            // Find and show preferred encoding
            if let Some(preferred) = accept_encoding.preferred() {
                response_lines.push("".to_string());
                response_lines.push(format!("Preferred encoding: {}", preferred));
            }

            // Show sorted preferences
            let mut sorted_accept = accept_encoding.clone();
            sorted_accept.sort_descending();

            response_lines.push("".to_string());
            response_lines.push("Sorted by preference (descending):".to_string());
            for (encoding, quality) in sorted_accept.items() {
                response_lines.push(format!("  {} (q={})", encoding, quality));
            }

            response_lines.join("\n")
        }
        None => {
            "No Accept-Encoding header provided\n\nTry: curl -H 'Accept-Encoding: gzip, deflate;q=0.8' http://localhost:3000/".to_string()
        }
    }
}

/// API handler with content negotiation and JSON response
async fn api_handler(
    State(state): State<AppState>,
    accept_encoding: Option<TypedHeader<AcceptEncoding>>,
) -> Response {
    let response_data = match accept_encoding.as_ref() {
        Some(TypedHeader(accept_encoding)) => {
            // Perform content negotiation
            let selected_encoding = negotiate_encoding(accept_encoding, &state.supported_encodings);

            ApiResponse {
                message: "Content negotiation successful!".to_string(),
                encoding_used: selected_encoding.to_string(),
                client_preferences: accept_encoding.items()
                    .iter()
                    .map(|(enc, q)| EncodingPreference {
                        encoding: enc.to_string(),
                        quality: *q,
                    })
                    .collect(),
                server_capabilities: state.supported_encodings
                    .iter()
                    .map(|e| e.to_string())
                    .collect(),
            }
        }
        None => ApiResponse {
            message: "No Accept-Encoding header provided".to_string(),
            encoding_used: "identity".to_string(),
            client_preferences: vec![],
            server_capabilities: state.supported_encodings
                .iter()
                .map(|e| e.to_string())
                .collect(),
        }
    };

    // Create response with appropriate Content-Encoding header
    let selected_encoding = response_data.encoding_used.parse::<Encoding>().unwrap_or(Encoding::Identity);
    let content_encoding = ContentEncoding::new(selected_encoding);

    let mut response = Json(response_data).into_response();

    // Add Content-Encoding header using TypedHeader
    if let Ok(header_value) = http::HeaderValue::from_str(&content_encoding.encoding().to_string()) {
        response.headers_mut().insert(
            http::header::CONTENT_ENCODING,
            header_value,
        );
    }

    response
}

/// Advanced handler demonstrating dynamic server configuration
async fn negotiate_handler(
    Query(config): Query<ServerConfig>,
    accept_encoding: Option<TypedHeader<AcceptEncoding>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Parse server-supported encodings from query parameter
    let server_encodings: Vec<Encoding> = config.supported
        .split(',')
        .filter_map(|s| s.trim().parse::<Encoding>().ok())
        .collect();

    if server_encodings.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut negotiation_result = serde_json::json!({
        "server_supported": server_encodings.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    });

    match accept_encoding {
        Some(TypedHeader(accept_encoding)) => {
            let selected = negotiate_encoding(&accept_encoding, &server_encodings);

            // Create detailed negotiation information
            let mut sorted_accept = accept_encoding.clone();
            sorted_accept.sort_descending();

            negotiation_result["negotiation"] = serde_json::json!({
                "selected_encoding": selected.to_string(),
                "client_preferences": sorted_accept.items()
                    .iter()
                    .map(|(enc, q)| serde_json::json!({
                        "encoding": enc.to_string(),
                        "quality": q,
                        "supported_by_server": server_encodings.contains(enc)
                    }))
                    .collect::<Vec<_>>(),
                "negotiation_process": {
                    "method": "Iterate client preferences in quality order",
                    "result": format!("Selected '{}' (first acceptable match)", selected)
                }
            });
        }
        None => {
            negotiation_result["negotiation"] = serde_json::json!({
                "selected_encoding": "identity",
                "reason": "No Accept-Encoding header provided"
            });
        }
    }

    Ok(Json(negotiation_result))
}

/// Content negotiation algorithm
fn negotiate_encoding(accept_encoding: &AcceptEncoding, server_supported: &[Encoding]) -> Encoding {
    // Create a copy and sort by preference (highest quality first)
    let mut sorted_accept = accept_encoding.clone();
    sorted_accept.sort_descending();

    // Find first acceptable encoding
    for (encoding, quality) in sorted_accept.items() {
        if *quality > 0.0 && server_supported.contains(encoding) {
            return encoding.clone();
        }
    }

    // Fallback to identity if no match found
    Encoding::Identity
}
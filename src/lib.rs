use axum::{
    body::Bytes,
    http::header::HeaderMap,
    http::{Method, Uri},
    Router,
};

// https://docs.rs/axum/latest/axum/extract/index.html#common-extractors
async fn echoreq(method: Method, uri: Uri, headers: HeaderMap, body: Bytes) -> String {
    // https://docs.rs/http/1.0.0/http/header/struct.HeaderMap.html
    let headers = headers
        .iter()
        .map(|(k, v)| {
            let v = v.to_str().unwrap_or_default();
            format!("{k}: {}", v)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let body = match String::from_utf8(body.to_vec()) {
        Ok(body) => body,
        Err(_) => printable_bytes(body),
    };

    println!("Received: {method} {uri}");

    format!("{method} {uri}\n{headers}\n\n{body}")
}

pub fn router() -> Router {
    Router::new().fallback(echoreq)
}

// TODO: eventually adapt some from std::core::str::validation::run_utf8_validation
fn printable_bytes(v: Bytes) -> String {
    let mut index = 0;
    let len = v.len();
    let mut out = vec!['.' as u8; len];

    while index < len {
        let oct = v[index];
        match oct {
            128.. => {
                // TODO: handle utf8 chars (oct >= 128), just validate them and copy bytes over -- must increment index extra
            }
            0..=8 | 11 | 12 | 14..=31 => {
                // just skip unprintable / invalid bytes (already '.')
            }
            _ => out[index] = oct,
        }
        index += 1;
    }

    unsafe { String::from_utf8_unchecked(out) }
}

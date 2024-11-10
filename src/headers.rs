use actix_web::HttpResponseBuilder;
use reqwest::{header::HeaderMap, Response};

pub fn add_headers(response: &mut HttpResponseBuilder) {
    response
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Access-Control-Allow-Headers", "*"))
        .append_header(("Access-Control-Allow-Methods", "*"))
        .append_header(("Access-Control-Max-Age", "1728000"));
}

pub fn is_header_allowed(header: &str) -> bool {
    if header.starts_with("access-control") {
        return false;
    }

    !matches!(
        header,
        "host"
            | "content-length"
            | "set-cookie"
            | "alt-svc"
            | "accept-ch"
            | "report-to"
            | "strict-transport-security"
            | "user-agent"
            | "range"
            | "transfer-encoding"
            | "x-real-ip"
            | "origin"
            | "referer"
            // the 'x-title' header contains non-ascii characters which is not allowed on some HTTP clients
            | "x-title"
    )
}

pub fn get_content_length(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("content-length")
        .and_then(|cl| cl.to_str().ok())
        .and_then(|cl| str::parse::<u64>(cl).ok())
}

pub fn copy_response_headers(req_resp: &Response, http_resp: &mut HttpResponseBuilder) {
    add_headers(http_resp);

    for (key, value) in req_resp.headers() {
        if is_header_allowed(key.as_str()) {
            http_resp.append_header((key.as_str(), value.as_bytes()));
        }
    }
}

use std::env;
use std::error::Error;

use actix_web::{App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, web};
use actix_web::http::Method;
use image::EncodableLayout;
use lazy_static::lazy_static;
use qstring::QString;
use regex::Regex;
use reqwest::{Client, Request, Url};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Running server!");

    let server =HttpServer::new(|| {
        // match all requests
         App::new()
            .default_service(web::to(index))
    });
    // get port from env
    if env::var("UDS").is_ok() {
        server.bind_uds("./socket/actix.sock")?
    } else {
        let bind = env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        server.bind(bind)?
    }.run().await
}

lazy_static!(
    static ref RE: Regex = Regex::new(r"^[a-z\d\.-]*$").unwrap();
    static ref RE_DOMAIN: Regex = Regex::new(r"(?:^[a-z\d\.-]*\.)?((?:[a-z\d-]*)\.(?:[a-z\d-]*))$").unwrap();
    static ref RE_MANIFEST: Regex = Regex::new("(?m)URI=\"([^\"]+)\"").unwrap();
);

lazy_static!(
    static ref CLIENT: Client = Client::new();
);

const ALLOWED_DOMAINS: [&str; 7] = [
    "youtube.com",
    "googlevideo.com",
    "ytimg.com",
    "ggpht.com",
    "googleusercontent.com",
    "lbryplayer.xyz",
    "odycdn.com",
];

fn add_headers(response: &mut HttpResponseBuilder) {
    response
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Access-Control-Allow-Headers", "*"))
        .append_header(("Access-Control-Allow-Methods", "*"))
        .append_header(("Access-Control-Max-Age", "1728000"));
}

fn is_header_allowed(header: &str) -> bool {
    if header.starts_with("access-control") {
        return false;
    }

    match header {
        "host" | "content-length" | "set-cookie" | "alt-svc" | "accept-ch" | "report-to" | "strict-transport-security" => false,
        _ => true,
    }
}

async fn index(req: HttpRequest) -> Result<HttpResponse, Box<dyn Error>> {
    if req.method() == Method::OPTIONS {
        let mut response = HttpResponse::Ok();
        add_headers(&mut response);
        return Ok(response.finish());
    } else if req.method() != Method::GET && req.method() != Method::HEAD {
        let mut response = HttpResponse::MethodNotAllowed();
        add_headers(&mut response);
        return Ok(response.finish());
    }

    // parse query string
    let query = QString::from(req.query_string());

    let res = query.get("host");

    if res.is_none() {
        return Err("No host provided".into());
    }

    let host = res.unwrap();

    if !RE.is_match(host) || !RE_DOMAIN.is_match(host) {
        return Err("Invalid host provided".into());
    }

    let domain = RE_DOMAIN.captures(host).unwrap().get(1).unwrap().as_str();

    let mut allowed = false;

    for allowed_domain in ALLOWED_DOMAINS.iter() {
        if &domain == allowed_domain {
            allowed = true;
            break;
        }
    }

    if !allowed {
        return Err("Domain not allowed".into());
    }

    let mut url = Url::parse(&*format!("https://{}{}", host, req.path()))?;
    url.set_query(Some(req.query_string()));

    let mut request = Request::new(
        req.method().clone(),
        url,
    );

    let request_headers = request.headers_mut();

    for (key, value) in req.headers() {
        if is_header_allowed(key.as_str()) {
            request_headers.insert(key.clone(), value.clone());
        }
    }

    let resp = CLIENT.execute(request).await;

    if resp.is_err() {
        return Err(resp.err().unwrap().into());
    }

    let resp = resp.unwrap();

    let mut response = HttpResponse::build(resp.status());

    add_headers(&mut response);

    for (key, value) in resp.headers() {
        if is_header_allowed(key.as_str()) {
            response.append_header((key.as_str(), value.to_str().unwrap()));
        }
    }

    let content_type = resp.headers().get("content-type");

    if content_type.is_some() {
        let content_type = content_type.unwrap();
        if content_type == "image/jpeg" {
            let resp_bytes = resp.bytes().await.unwrap();

            let image = image::load_from_memory(&resp_bytes).unwrap();

            let encoder = webp::Encoder::from_image(&image).unwrap();

            let encoded = encoder.encode(85f32);
            let bytes = encoded.as_bytes().to_vec();

            if bytes.len() < resp_bytes.len() {
                response.content_type("image/webp");
                return Ok(response.body(bytes));
            }

            return Ok(response.body(resp_bytes));
        }
        if content_type == "application/x-mpegurl" || content_type == "application/vnd.apple.mpegurl" {
            let resp_str = resp.text().await.unwrap();

            let modified = resp_str.lines().map(|line| {
                let captures = RE_MANIFEST.captures(line);
                if captures.is_some() {
                    let url = captures.unwrap().get(1).unwrap().as_str();
                    if url.starts_with("https://") {
                        return line.replace(url, localize_url(url, host).as_str());
                    }
                } else if line.starts_with("https://") {
                    return localize_url(line, host);
                }
                line.to_string()
            }).collect::<Vec<String>>().join("\n");

            return Ok(response.body(modified));
        }
    }

    // Stream response
    Ok(response.streaming(resp.bytes_stream()))
}

fn localize_url(url: &str, host: &str) -> String {
    if url.starts_with("https://") {
        let mut url = Url::parse(url).unwrap();
        let host = url.host().unwrap().to_string();

        // set host query param
        url.query_pairs_mut()
            .append_pair("host", &host);

        return format!("{}?{}", url.path(), url.query().unwrap());
    } else if url.starts_with("/") {
        return if url.contains("?") {
            format!("{}&host={}", url, host)
        } else {
            format!("{}?host={}", url, host)
        }
    }

    url.to_string()
}

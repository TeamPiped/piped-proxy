use std::env;
use std::error::Error;

use actix_web::{App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, web};
use actix_web::http::Method;
use libwebp_sys::{WebPEncodeRGB, WebPFree};
use mimalloc::MiMalloc;
use once_cell::sync::Lazy;
use qstring::QString;
use ravif::{Encoder, Img};
use regex::Regex;
use reqwest::{Body, Client, Request, Url};
use rgb::FromSlice;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Running server!");

    let server = HttpServer::new(|| {
        // match all requests
        App::new().default_service(web::to(index))
    });
    // get port from env
    if env::var("UDS").is_ok() {
        server.bind_uds("./socket/actix.sock")?
    } else {
        let bind = env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        server.bind(bind)?
    }
        .run()
        .await
}

static RE_DOMAIN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:[a-z\d.-]*\.)?([a-z\d-]*\.[a-z\d-]*)$").unwrap());
static RE_MANIFEST: Lazy<Regex> = Lazy::new(|| Regex::new("(?m)URI=\"([^\"]+)\"").unwrap());
static RE_DASH_MANIFEST: Lazy<Regex> =
    Lazy::new(|| Regex::new("BaseURL>(https://[^<]+)</BaseURL").unwrap());

static CLIENT: Lazy<Client> = Lazy::new(|| {
    let builder = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0");

    if env::var("IPV4_ONLY").is_ok() {
        builder
            .local_address(Some("0.0.0.0".parse().unwrap()))
            .build()
            .unwrap()
    } else {
        builder.build().unwrap()
    }
});

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
    )
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
    let res = res.map(|s| s.to_string());

    if res.is_none() {
        return Err("No host provided".into());
    }

    let rewrite = {
        if let Some(rewrite) = query.get("rewrite") {
            rewrite == "true"
        } else {
            true
        }
    };

    let avif = {
        if let Some(avif) = query.get("avif") {
            avif == "true"
        } else {
            false
        }
    };

    let host = res.unwrap();
    let domain = RE_DOMAIN.captures(host.as_str());

    if domain.is_none() {
        return Err("Invalid host provided".into());
    }

    let domain = domain.unwrap().get(1).unwrap().as_str();

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

    let video_playback = req.path().eq("/videoplayback");
    let is_android = video_playback && query.get("c").unwrap_or("").eq("ANDROID");

    let qs = {
        let collected = query
            .into_pairs()
            .into_iter()
            .filter(|(key, _)| key != "host" && key != "rewrite")
            .collect::<Vec<_>>();
        QString::new(collected)
    };

    let mut url = Url::parse(&format!("https://{}{}", host, req.path()))?;
    url.set_query(Some(qs.to_string().as_str()));

    let method = {
        if !is_android && video_playback {
            Method::POST
        } else {
            req.method().clone()
        }
    };

    let mut request = Request::new(method, url);

    if !is_android && video_playback {
        request.body_mut().replace(Body::from("x\0"));
    }

    let request_headers = request.headers_mut();

    for (key, value) in req.headers() {
        if is_header_allowed(key.as_str()) {
            request_headers.insert(key, value.clone());
        }
    }

    if is_android {
        request_headers.insert("User-Agent", "com.google.android.youtube/1537338816 (Linux; U; Android 13; en_US; ; Build/TQ2A.230505.002; Cronet/113.0.5672.24)".parse().unwrap());
    }

    let resp = CLIENT.execute(request).await;

    if resp.is_err() {
        return Err(resp.err().unwrap().into());
    }

    let resp = resp?;

    let mut response = HttpResponse::build(resp.status());

    add_headers(&mut response);

    for (key, value) in resp.headers() {
        if is_header_allowed(key.as_str()) {
            response.append_header((key.as_str(), value.to_str()?));
        }
    }

    if rewrite {
        if let Some(content_type) = resp.headers().get("content-type") {
            if content_type == "image/webp" || content_type == "image/jpeg" && avif {
                let resp_bytes = resp.bytes().await.unwrap();

                let image = image::load_from_memory(&resp_bytes).unwrap();

                let buffer = Img::new(
                    image.as_rgb8().unwrap().as_raw().as_rgb(),
                    image.width() as usize,
                    image.height() as usize,
                );

                let res = Encoder::new()
                    .with_quality(80f32)
                    .with_speed(4)
                    .encode_rgb(buffer);

                return if let Ok(res) = res {
                    response.content_type("image/avif");
                    Ok(response.body(res.avif_file.to_vec()))
                } else {
                    response.content_type("image/jpeg");
                    Ok(response.body(resp_bytes))
                };
            }

            if content_type == "image/jpeg" {
                let resp_bytes = resp.bytes().await.unwrap();

                let image = image::load_from_memory(&resp_bytes).unwrap();
                let width = image.width();
                let height = image.height();

                let quality = 85;

                let data = image.as_rgb8().unwrap().as_raw();

                let bytes: Vec<u8> = unsafe {
                    let mut out_buf = std::ptr::null_mut();
                    let stride = width as i32 * 3;
                    let len: usize = WebPEncodeRGB(
                        data.as_ptr(),
                        width as i32,
                        height as i32,
                        stride,
                        quality as f32,
                        &mut out_buf,
                    );
                    let vec = std::slice::from_raw_parts(out_buf, len).into();
                    WebPFree(out_buf as *mut _);
                    vec
                };

                if bytes.len() < resp_bytes.len() {
                    response.content_type("image/webp");
                    return Ok(response.body(bytes));
                }

                response.content_type("image/jpeg");
                return Ok(response.body(resp_bytes));
            }
            if content_type == "application/x-mpegurl"
                || content_type == "application/vnd.apple.mpegurl"
            {
                let resp_str = resp.text().await.unwrap();

                let modified = resp_str
                    .lines()
                    .map(|line| {
                        let captures = RE_MANIFEST.captures(line);
                        if let Some(captures) = captures {
                            let url = captures.get(1).unwrap().as_str();
                            if url.starts_with("https://") {
                                return line
                                    .replace(url, localize_url(url, host.as_str()).as_str());
                            }
                        }
                        localize_url(line, host.as_str())
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                return Ok(response.body(modified));
            }
            if content_type == "video/vnd.mpeg.dash.mpd" || content_type == "application/dash+xml" {
                let mut resp_str = resp.text().await.unwrap();
                let clone_resp = resp_str.clone();
                let captures = RE_DASH_MANIFEST.captures_iter(&clone_resp);
                for capture in captures {
                    let url = capture.get(1).unwrap().as_str();
                    let new_url = localize_url(url, host.as_str());
                    resp_str = resp_str.replace(url, new_url.as_str());
                }
                return Ok(response.body(resp_str));
            }
        }
    }

    if let Some(content_length) = resp.headers().get("content-length") {
        response.append_header(("content-length", content_length));
    }

    // Stream response
    Ok(response.streaming(resp.bytes_stream()))
}

fn localize_url(url: &str, host: &str) -> String {
    if url.starts_with("https://") {
        let mut url = Url::parse(url).unwrap();
        let host = url.host().unwrap().to_string();

        // set host query param
        url.query_pairs_mut().append_pair("host", &host);

        return format!("{}?{}", url.path(), url.query().unwrap());
    } else if url.ends_with(".m3u8") || url.ends_with(".ts") {
        return if url.contains('?') {
            format!("{}&host={}", url, host)
        } else {
            format!("{}?host={}", url, host)
        };
    }

    url.to_string()
}

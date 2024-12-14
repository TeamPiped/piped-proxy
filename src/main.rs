mod ump_stream;
mod utils;

use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer};
use listenfd::ListenFd;
use once_cell::sync::Lazy;
use qstring::QString;
use regex::Regex;
use reqwest::{Body, Client, Request, Url};
use std::error::Error;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};

#[cfg(not(any(feature = "reqwest-native-tls", feature = "reqwest-rustls")))]
compile_error!("feature \"reqwest-native-tls\" or \"reqwest-rustls\" must be set for proxy to have TLS support");

use futures_util::TryStreamExt;
use http::{HeaderName, Method};
use reqwest::header::HeaderValue;
#[cfg(any(feature = "webp", feature = "avif", feature = "qhash"))]
use tokio::task::spawn_blocking;
use ump_stream::UmpTransformStream;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn try_get_fd_listeners() -> (Option<UnixListener>, Option<TcpListener>) {
    let mut fd = ListenFd::from_env();

    let unix_listener = env::var("FD_UNIX").ok().map(|fd_unix| {
        let fd_pos = fd_unix.parse().expect("FD_UNIX is not a number");
        println!("Trying to take Unix socket at position {}", fd_pos);
        fd.take_unix_listener(fd_pos)
            .expect(format!("fd {} is not a Unix socket", fd_pos).as_str())
            .expect(format!("fd {} has already been used", fd_pos).as_str())
    });

    let tcp_listener = env::var("FD_TCP").ok().map(|fd_tcp| {
        let fd_pos = fd_tcp.parse().expect("FD_TCP is not a number");
        println!("Trying to take TCP listener at position {}", fd_pos);
        fd.take_tcp_listener(fd_pos)
            .expect(format!("fd {} is not a TCP listener", fd_pos).as_str())
            .expect(format!("fd {} has already been used", fd_pos).as_str())
    });

    (unix_listener, tcp_listener)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Running server!");

    let mut server = HttpServer::new(|| {
        // match all requests
        App::new().default_service(web::to(index))
    });

    let fd_listeners = try_get_fd_listeners();

    if let Some(unix_listener) = fd_listeners.0 {
        server = server
            .listen_uds(unix_listener)
            .expect("Error while trying to listen on Unix socket passed by fd");
        println!("Listening on Unix socket passed by fd.");
    }

    if let Some(tcp_listener) = fd_listeners.1 {
        server = server
            .listen(tcp_listener)
            .expect("Error while trying to listen on TCP listener passed by fd");
        println!("Listening on TCP listener passed by fd.");
    }

    // Only bind manually if there is not already a listener
    if server.addrs().is_empty() {
        // get socket/port from env
        // backwards compat when only UDS is set
        server = if utils::get_env_bool("UDS") {
            let socket_path =
                env::var("BIND_UNIX").unwrap_or_else(|_| "./socket/actix.sock".to_string());
            server.bind_uds(socket_path)?
        } else {
            let bind = env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
            server.bind(bind)?
        };
    }

    server.run().await
}

static RE_DOMAIN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:[a-z\d.-]*\.)?([a-z\d-]*\.[a-z\d-]*)$").unwrap());
static RE_MANIFEST: Lazy<Regex> = Lazy::new(|| Regex::new("(?m)URI=\"([^\"]+)\"").unwrap());
static RE_DASH_MANIFEST: Lazy<Regex> =
    Lazy::new(|| Regex::new("BaseURL>(https://[^<]+)</BaseURL").unwrap());

static CLIENT: Lazy<Client> = Lazy::new(|| {
    let builder = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0");

    let proxy = if let Ok(proxy) = env::var("PROXY") {
        reqwest::Proxy::all(proxy).ok()
    } else {
        None
    };

    let builder = if let Some(proxy) = proxy {
        // proxy basic auth
        if let Ok(proxy_auth_user) = env::var("PROXY_USER") {
            let proxy_auth_pass = env::var("PROXY_PASS").unwrap_or_default();
            builder.proxy(proxy.basic_auth(&proxy_auth_user, &proxy_auth_pass))
        } else {
            builder.proxy(proxy)
        }
    } else {
        builder
    };

    if utils::get_env_bool("IPV4_ONLY") {
        builder.local_address("0.0.0.0".parse().ok())
    } else {
        builder
    }
    .build()
    .unwrap()
});

const ANDROID_USER_AGENT: &str = "com.google.android.youtube/1537338816 (Linux; U; Android 13; en_US; ; Build/TQ2A.230505.002; Cronet/113.0.5672.24)";
const ALLOWED_DOMAINS: [&str; 8] = [
    "youtube.com",
    "googlevideo.com",
    "ytimg.com",
    "ggpht.com",
    "googleusercontent.com",
    "lbryplayer.xyz",
    "odycdn.com",
    "ajay.app",
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
            | "range"
            | "transfer-encoding"
            | "x-real-ip"
            | "origin"
            | "referer"
            // the 'x-title' header contains non-ascii characters which is not allowed on some HTTP clients
            | "x-title"
    )
}

async fn index(req: HttpRequest) -> Result<HttpResponse, Box<dyn Error>> {
    if req.method() == actix_web::http::Method::OPTIONS {
        let mut response = HttpResponse::Ok();
        add_headers(&mut response);
        return Ok(response.finish());
    } else if req.method() != actix_web::http::Method::GET
        && req.method() != actix_web::http::Method::HEAD
    {
        let mut response = HttpResponse::MethodNotAllowed();
        add_headers(&mut response);
        return Ok(response.finish());
    }

    // parse query string
    let mut query = QString::from(req.query_string());

    #[cfg(feature = "qhash")]
    {
        use std::collections::BTreeSet;

        let secret = env::var("HASH_SECRET");
        if let Ok(secret) = secret {
            let Some(qhash) = query.get("qhash") else {
                return Err("No qhash provided".into());
            };

            if qhash.len() != 8 {
                return Err("Invalid qhash provided".into());
            }

            let path = req.path().as_bytes().to_owned();

            // Store sorted key-value pairs
            let mut set = BTreeSet::new();
            {
                let pairs = query.to_pairs();
                for (key, value) in &pairs {
                    if matches!(*key, "qhash" | "range" | "rewrite") {
                        continue;
                    }
                    set.insert((key.as_bytes().to_owned(), value.as_bytes().to_owned()));
                }
            }

            let hash = spawn_blocking(move || {
                let mut hasher = blake3::Hasher::new();

                for (key, value) in set {
                    hasher.update(&key);
                    hasher.update(&value);
                }

                let range_marker = b"/range/";

                // Find the slice before "/range/"
                if let Some(position) = path
                    .windows(range_marker.len())
                    .position(|window| window == range_marker)
                {
                    // Update the hasher with the part of the path before "/range/"
                    // We add +1 to include the "/" in the hash
                    // This is done for DASH streams for the manifests provided by YouTube
                    hasher.update(&path[..(position + 1)]);
                } else {
                    hasher.update(&path);
                }

                hasher.update(secret.as_bytes());

                let hash = hasher.finalize().to_hex();

                hash[..8].to_owned()
            })
            .await
            .unwrap();

            if hash != qhash {
                return Err("Invalid qhash provided".into());
            }
        }
    }

    let Some(host) = query.get("host").map(|s| s.to_string()) else {
        return Err("No host provided".into());
    };

    #[cfg(any(feature = "webp", feature = "avif"))]
    let disallow_image_transcoding = utils::get_env_bool("DISALLOW_IMAGE_TRANSCODING");

    let rewrite = query.get("rewrite") != Some("false");

    #[cfg(feature = "avif")]
    let avif = query.get("avif") == Some("true");

    let Some(domain) = RE_DOMAIN
        .captures(host.as_str())
        .map(|domain| domain.get(1).unwrap().as_str())
    else {
        return Err("Invalid host provided".into());
    };

    if !ALLOWED_DOMAINS.contains(&domain) {
        return Err("Domain not allowed".into());
    }

    let video_playback = req.path().eq("/videoplayback");

    if video_playback {
        if let Some(expiry) = query.get("expire") {
            let expiry = expiry.parse::<i64>()?;
            let now = SystemTime::now();
            let now = now.duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64;
            if now < expiry {
                return Err("Expire time in past".into());
            }
        }
    }

    let is_android = video_playback && query.get("c").unwrap_or("").eq("ANDROID");
    let is_web = video_playback && query.get("c").unwrap_or("").eq("WEB");

    let is_ump = video_playback && query.get("ump").is_some();

    let mime_type = query.get("mime").map(|s| s.to_string());

    let clen = query
        .get("clen")
        .map(|s| s.to_string().parse::<u64>().unwrap());

    if video_playback && !query.has("range") {
        if let Some(range) = req.headers().get("range") {
            let range = range.to_str().unwrap();
            let range = range.replace("bytes=", "");
            let range = range.split('-').collect::<Vec<_>>();
            let start = range[0].parse::<u64>().unwrap();
            let end = match range[1].parse::<u64>() {
                Ok(end) => end,
                Err(_) => {
                    if let Some(clen) = clen {
                        clen - 1
                    } else {
                        0
                    }
                }
            };
            if end != 0 {
                let range = format!("{}-{}", start, end);
                query.add_pair(("range", range));
            }
        } else if let Some(clen) = clen {
            let range = format!("0-{}", clen - 1);
            query.add_pair(("range", range));
        }
    }

    let range = query.get("range").map(|s| s.to_string());

    let qs = {
        let collected = query
            .into_pairs()
            .into_iter()
            .filter(|(key, _)| !matches!(key.as_str(), "host" | "rewrite" | "qhash"))
            .collect::<Vec<_>>();
        QString::new(collected)
    };

    let mut url = Url::parse(&format!("https://{}{}", host, req.path()))?;
    url.set_query(Some(qs.to_string().as_str()));

    let method = {
        if is_web && video_playback {
            Method::POST
        } else {
            Method::from_str(req.method().as_str())?
        }
    };

    let mut request = Request::new(method, url);

    if is_web && video_playback {
        request.body_mut().replace(Body::from("x\0"));
    }

    let request_headers = request.headers_mut();

    for (key, value) in req.headers() {
        let key = key.as_str();
        if is_header_allowed(key) {
            request_headers.insert(
                HeaderName::from_str(key)?,
                HeaderValue::from_bytes(value.as_bytes())?,
            );
        }
    }

    if is_android {
        request_headers.insert("User-Agent", ANDROID_USER_AGENT.parse()?);
    }

    let resp = CLIENT.execute(request).await?;

    let mut response = HttpResponse::build(StatusCode::from_u16(resp.status().as_u16())?);

    add_headers(&mut response);

    for (key, value) in resp.headers() {
        if is_header_allowed(key.as_str()) {
            response.append_header((key.as_str(), value.as_bytes()));
        }
    }

    if rewrite {
        if let Some(content_type) = resp.headers().get("content-type") {
            #[cfg(feature = "avif")]
            if !disallow_image_transcoding
                && (content_type == "image/webp" || content_type == "image/jpeg" && avif)
            {
                let resp_bytes = resp.bytes().await.unwrap();
                let (body, content_type) = spawn_blocking(|| {
                    use ravif::{Encoder, Img};
                    use rgb::FromSlice;

                    let image = image::load_from_memory(&resp_bytes).unwrap();

                    let width = image.width() as usize;
                    let height = image.height() as usize;

                    let buf = image.into_rgb8();
                    let buf = buf.as_raw().as_rgb();

                    let buffer = Img::new(buf, width, height);

                    let res = Encoder::new()
                        .with_quality(80f32)
                        .with_speed(7)
                        .encode_rgb(buffer);

                    if let Ok(res) = res {
                        (res.avif_file.to_vec(), "image/avif")
                    } else {
                        (resp_bytes.into(), "image/jpeg")
                    }
                })
                .await
                .unwrap();
                response.content_type(content_type);
                return Ok(response.body(body));
            }

            #[cfg(feature = "webp")]
            if !disallow_image_transcoding && content_type == "image/jpeg" {
                let resp_bytes = resp.bytes().await.unwrap();
                let (body, content_type) = spawn_blocking(|| {
                    use libwebp_sys::{WebPEncodeRGB, WebPFree};

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
                        (bytes, "image/webp")
                    } else {
                        (resp_bytes.into(), "image/jpeg")
                    }
                })
                .await
                .unwrap();
                response.content_type(content_type);
                return Ok(response.body(body));
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
                                return line.replace(
                                    url,
                                    utils::localize_url(url, host.as_str()).as_str(),
                                );
                            }
                        }
                        utils::localize_url(line, host.as_str())
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                return Ok(response.body(modified));
            }
            if content_type == "video/vnd.mpeg.dash.mpd" || content_type == "application/dash+xml" {
                let resp_str = resp.text().await.unwrap();
                let mut new_resp = resp_str.clone();
                let captures = RE_DASH_MANIFEST.captures_iter(&resp_str);
                for capture in captures {
                    let url = capture.get(1).unwrap().as_str();
                    let new_url = utils::localize_url(url, host.as_str());
                    let new_url = utils::escape_xml(new_url.as_str());
                    new_resp = new_resp.replace(url, new_url.as_ref());
                }
                return Ok(response.body(new_resp));
            }
        }
    }

    if let Some(content_length) = resp.headers().get("content-length") {
        response.no_chunking(content_length.to_str().unwrap().parse::<u64>().unwrap());
    }

    if is_ump && resp.status().is_success() {
        if let Some(mime_type) = mime_type {
            response.content_type(mime_type);
        }
        if req.headers().contains_key("range") {
            // check if it's not the whole stream
            if let Some(ref range) = range {
                if let Some(clen) = clen {
                    if range != &format!("0-{}", clen - 1) {
                        response.status(StatusCode::PARTIAL_CONTENT);
                    }
                }
            }
        }
        let resp = resp.bytes_stream();
        let resp = resp.map_err(|e| io::Error::new(ErrorKind::Other, e));
        let transformed_stream = UmpTransformStream::new(resp);
        // print errors
        let transformed_stream = transformed_stream.map_err(|e| {
            eprintln!("UMP Transforming Error: {}", e);
            e
        });

        // calculate content length from clen and range
        if let Some(clen) = clen {
            let length = if let Some(ref range) = range {
                let range = range.replace("bytes=", "");
                let range = range.split('-').collect::<Vec<_>>();
                let start = range[0].parse::<u64>().unwrap();
                let end = range[1].parse::<u64>().unwrap_or(clen - 1);
                end - start + 1
            } else {
                clen
            };
            response.no_chunking(length);
        }

        return Ok(response.streaming(transformed_stream));
    }

    // Stream response
    Ok(response.streaming(resp.bytes_stream()))
}

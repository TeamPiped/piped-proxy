use actix_web::http::{Method, StatusCode};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer};
use once_cell::sync::Lazy;
use qstring::QString;
use regex::Regex;
use reqwest::Error as ReqwestError;
use reqwest::{Body, Client, Request, Url};
use std::collections::BTreeMap;
use std::error::Error;
use std::io::ErrorKind;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{env, io};

#[cfg(not(any(feature = "reqwest-native-tls", feature = "reqwest-rustls")))]
compile_error!("feature \"reqwest-native-tls\" or \"reqwest-rustls\" must be set for proxy to have TLS support");

use bytes::{Bytes, BytesMut};
use futures_util::Stream;
#[cfg(any(feature = "webp", feature = "avif", feature = "qhash"))]
use tokio::task::spawn_blocking;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Running server!");

    let server = HttpServer::new(|| {
        // match all requests
        App::new().default_service(web::to(index))
    });

    // get socket/port from env
    // backwards compat when only UDS is set
    if get_env_bool("UDS") {
        let socket_path =
            env::var("BIND_UNIX").unwrap_or_else(|_| "./socket/actix.sock".to_string());
        server.bind_uds(socket_path)?
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

    let proxy = if let Ok(proxy) = env::var("PROXY") {
        Some(reqwest::Proxy::all(proxy).unwrap())
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

    if get_env_bool("IPV4_ONLY") {
        builder
            .local_address(Some("0.0.0.0".parse().unwrap()))
            .build()
            .unwrap()
    } else {
        builder.build().unwrap()
    }
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
    )
}

fn get_env_bool(key: &str) -> bool {
    match env::var(key) {
        Ok(val) => val.to_lowercase() == "true" || val == "1",
        Err(_) => false,
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
    let mut query = QString::from(req.query_string());

    #[cfg(feature = "qhash")]
    {
        use std::collections::BTreeSet;

        let secret = env::var("HASH_SECRET");
        if let Ok(secret) = secret {
            let qhash = query.get("qhash");

            if qhash.is_none() {
                return Err("No qhash provided".into());
            }

            let qhash = qhash.unwrap();

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

                hasher.update(&path);

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

    let res = query.get("host");
    let res = res.map(|s| s.to_string());

    if res.is_none() {
        return Err("No host provided".into());
    }

    #[cfg(any(feature = "webp", feature = "avif"))]
    let disallow_image_transcoding = get_env_bool("DISALLOW_IMAGE_TRANSCODING");

    let rewrite = query.get("rewrite") != Some("false");

    #[cfg(feature = "avif")]
    let avif = query.get("avif") == Some("true");

    let host = res.unwrap();
    let domain = RE_DOMAIN.captures(host.as_str());

    if domain.is_none() {
        return Err("Invalid host provided".into());
    }

    let domain = domain.unwrap().get(1).unwrap().as_str();

    if !ALLOWED_DOMAINS.contains(&domain) {
        return Err("Domain not allowed".into());
    }

    let video_playback = req.path().eq("/videoplayback");
    let is_android = video_playback && query.get("c").unwrap_or("").eq("ANDROID");

    let is_ump = video_playback && query.get("ump").is_some();

    let mime_type = query.get("mime").map(|s| s.to_string());

    if is_ump && !query.has("range") {
        if let Some(range) = req.headers().get("range") {
            let range = range.to_str().unwrap();
            let range = range.replace("bytes=", "");
            query.add_pair(("range", range));
        }
    }

    let has_range = query.has("range");

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
        request_headers.insert("User-Agent", ANDROID_USER_AGENT.parse().unwrap());
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
                let resp_str = resp.text().await.unwrap();
                let mut new_resp = resp_str.clone();
                let captures = RE_DASH_MANIFEST.captures_iter(&resp_str);
                for capture in captures {
                    let url = capture.get(1).unwrap().as_str();
                    let new_url = localize_url(url, host.as_str());
                    new_resp = new_resp.replace(url, new_url.as_str());
                }
                return Ok(response.body(new_resp));
            }
        }
    }

    if let Some(content_length) = resp.headers().get("content-length") {
        response.append_header(("content-length", content_length));
    }

    let resp = resp.bytes_stream();

    if is_ump {
        if let Some(mime_type) = mime_type {
            response.content_type(mime_type);
        }
        if has_range {
            response.status(StatusCode::PARTIAL_CONTENT);
        }
        let transformed_stream = UmpTransformStream::new(resp);
        return Ok(response.streaming(transformed_stream));
    }

    // Stream response
    Ok(response.streaming(resp))
}

fn read_buf(buf: &[u8], pos: &mut usize) -> u8 {
    let byte = buf[*pos];
    *pos += 1;
    byte
}

fn read_variable_integer(buf: &[u8], offset: usize) -> io::Result<(i32, usize)> {
    let mut pos = offset;
    let prefix = read_buf(buf, &mut pos);
    let mut size = 0;
    for shift in 1..=5 {
        if prefix & (128 >> (shift - 1)) == 0 {
            size = shift;
            break;
        }
    }
    if !(1..=5).contains(&size) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Invalid integer size {} at position {}", size, offset),
        ));
    }

    match size {
        1 => Ok((prefix as i32, size)),
        2 => {
            let value = ((read_buf(buf, &mut pos) as i32) << 6) | (prefix as i32 & 0b111111);
            Ok((value, size))
        }
        3 => {
            let value =
                (((read_buf(buf, &mut pos) as i32) | ((read_buf(buf, &mut pos) as i32) << 8)) << 5)
                    | (prefix as i32 & 0b11111);
            Ok((value, size))
        }
        4 => {
            let value = (((read_buf(buf, &mut pos) as i32)
                | ((read_buf(buf, &mut pos) as i32) << 8)
                | ((read_buf(buf, &mut pos) as i32) << 16))
                << 4)
                | (prefix as i32 & 0b1111);
            Ok((value, size))
        }
        _ => {
            let value = (read_buf(buf, &mut pos) as i32)
                | ((read_buf(buf, &mut pos) as i32) << 8)
                | ((read_buf(buf, &mut pos) as i32) << 16)
                | ((read_buf(buf, &mut pos) as i32) << 24);
            Ok((value, size))
        }
    }
}

struct UmpTransformStream<S>
where
    S: Stream<Item = Result<Bytes, ReqwestError>> + Unpin,
{
    inner: S,
    buffer: BytesMut,
    found_stream: bool,
    remaining: usize,
}

impl<S> UmpTransformStream<S>
where
    S: Stream<Item = Result<Bytes, ReqwestError>> + Unpin,
{
    pub fn new(stream: S) -> Self {
        UmpTransformStream {
            inner: stream,
            buffer: BytesMut::new(),
            found_stream: false,
            remaining: 0,
        }
    }
}

impl<S> Stream for UmpTransformStream<S>
where
    S: Stream<Item = Result<Bytes, ReqwestError>> + Unpin,
{
    type Item = Result<Bytes, ReqwestError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        while let Poll::Ready(item) = Pin::new(&mut this.inner).poll_next(cx) {
            match item {
                Some(Ok(bytes)) => {
                    if this.found_stream {
                        if this.remaining > 0 {
                            let len = std::cmp::min(this.remaining, bytes.len());
                            this.remaining -= len;
                            if this.remaining == 0 {
                                this.buffer.clear();
                                this.buffer.extend_from_slice(&bytes[len..]);
                                this.found_stream = false;
                            }
                            return Poll::Ready(Some(Ok(bytes.slice(0..len))));
                        } else {
                            this.found_stream = false;
                            this.buffer.clear();
                            this.buffer.extend_from_slice(&bytes);
                        };
                    } else {
                        this.buffer.extend_from_slice(&bytes);
                    }
                }
                Some(Err(e)) => return Poll::Ready(Some(Err(e))),
                None => {
                    return Poll::Ready(None);
                }
            }
        }

        if !this.found_stream && !this.buffer.is_empty() {
            let (segment_type, s1) = read_variable_integer(&this.buffer, 0).unwrap();
            let (segment_length, s2) = read_variable_integer(&this.buffer, s1).unwrap();
            if segment_type != 21 {
                // Not the stream
                if this.buffer.len() > s1 + s2 + segment_length as usize {
                    let _ = this.buffer.split_to(s1 + s2 + segment_length as usize);
                }
            } else {
                this.remaining = segment_length as usize - 1;

                let _ = this.buffer.split_to(s1 + s2 + 1);

                if this.buffer.len() > segment_length as usize {
                    let len = std::cmp::min(this.remaining, this.buffer.len());
                    this.remaining -= len;

                    return Poll::Ready(Some(Ok(this.buffer.split_to(len).into())));
                } else {
                    this.remaining -= this.buffer.len();
                    this.found_stream = true;

                    return Poll::Ready(Some(Ok(this.buffer.to_vec().into())));
                }
            }
        }

        Poll::Pending
    }
}

fn finalize_url(path: &str, query: BTreeMap<String, String>) -> String {
    #[cfg(feature = "qhash")]
    {
        use std::collections::BTreeSet;

        let qhash = {
            let secret = env::var("HASH_SECRET");
            if let Ok(secret) = secret {
                let set = query
                    .iter()
                    .filter(|(key, _)| !matches!(key.as_str(), "qhash" | "range" | "rewrite"))
                    .map(|(key, value)| (key.as_bytes().to_owned(), value.as_bytes().to_owned()))
                    .collect::<BTreeSet<_>>();

                let mut hasher = blake3::Hasher::new();

                for (key, value) in set {
                    hasher.update(&key);
                    hasher.update(&value);
                }

                hasher.update(path.as_bytes());

                hasher.update(secret.as_bytes());

                let hash = hasher.finalize().to_hex();

                Some(hash[..8].to_owned())
            } else {
                None
            }
        };

        if qhash.is_some() {
            let mut query = QString::new(query.into_iter().collect::<Vec<_>>());
            query.add_pair(("qhash", qhash.unwrap()));
            return format!("{}?{}", path, query);
        }
    }

    let query = QString::new(query.into_iter().collect::<Vec<_>>());
    format!("{}?{}", path, query)
}

fn localize_url(url: &str, host: &str) -> String {
    if url.starts_with("https://") {
        let url = Url::parse(url).unwrap();
        let host = url.host().unwrap().to_string();

        let mut query = url.query_pairs().into_owned().collect::<BTreeMap<_, _>>();

        query.insert("host".to_string(), host.clone());

        return finalize_url(url.path(), query);
    } else if url.ends_with(".m3u8") || url.ends_with(".ts") {
        let mut query = BTreeMap::new();
        query.insert("host".to_string(), host.to_string());

        return finalize_url(url, query);
    }

    url.to_string()
}

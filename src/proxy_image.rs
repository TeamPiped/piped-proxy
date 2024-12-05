#[cfg(any(feature = "webp", feature = "avif"))]
use actix_web::Either;
use actix_web::{HttpRequest, HttpResponse};
use once_cell::sync::Lazy;
use reqwest::Url;

use crate::client::{create_request, CLIENT};
use crate::headers::{copy_response_headers, get_content_length};
#[cfg(any(feature = "webp", feature = "avif"))]
use crate::transcode_image::{transcode_image, DISALLOW_IMAGE_TRANSCODING};

pub async fn proxy(
    req: HttpRequest,
    src: ImageSource,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let mut resp = get_image(&req, src).await?;

    let mut response = HttpResponse::build(resp.status());

    copy_response_headers(&resp, &mut response);

    if !*DISALLOW_IMAGE_TRANSCODING {
        match transcode_image(
            resp,
            &mut response,
            #[cfg(feature = "avif")]
            true,
        )
        .await?
        {
            Either::Left(http_response) => return Ok(http_response),
            Either::Right(image_response) => resp = image_response,
        }
    }

    if let Some(content_length) = get_content_length(resp.headers()) {
        response.no_chunking(content_length);
    }

    Ok(response.streaming(resp.bytes_stream()))
}

#[derive(PartialEq)]
pub enum ImageSource {
    YtImg,
    GgPht,
}

impl ImageSource {
    fn get_base_url(&self) -> Url {
        static YTIMG_URL: Lazy<Url> =
            Lazy::new(|| Url::parse("https://i.ytimg.com").expect("Invalid ytimg URL"));
        static GGPHT_URL: Lazy<Url> =
            Lazy::new(|| Url::parse("https://yt3.ggpht.com").expect("Invalid ggpht URL"));

        match self {
            Self::YtImg => YTIMG_URL.clone(),
            Self::GgPht => GGPHT_URL.clone(),
        }
    }

    fn strip_path_prefix<'p>(&self, path: &'p str) -> &'p str {
        const GGPHT_PREFIX_LEN: usize = "/ggpht".len();

        match self {
            Self::YtImg => path,
            Self::GgPht => &path[GGPHT_PREFIX_LEN..],
        }
    }
}

const MAX_RES_SEGMENT: &str = "/maxres.jpg";

async fn get_image(req: &HttpRequest, src: ImageSource) -> reqwest::Result<reqwest::Response> {
    let req_uri = req.uri();

    let mut url = src.get_base_url();
    url.set_query(req_uri.query());

    let path = src.strip_path_prefix(req_uri.path());

    if src == ImageSource::YtImg && path.ends_with(MAX_RES_SEGMENT) {
        get_max_res_thumbnail(req, path, url).await
    } else {
        url.set_path(path);
        CLIENT
            .execute(create_request(req, req.method().clone(), url))
            .await
    }
}

async fn get_max_res_thumbnail(
    req: &HttpRequest,
    req_path: &str,
    mut proxy_url: Url,
) -> reqwest::Result<reqwest::Response> {
    const FORMATS: &[&str] = &["/maxresdefault.jpg", "/sddefault.jpg", "/hqdefault.jpg"];
    const DEFAULT_FORMAT: &str = "/mqdefault.jpg";
    const FORMAT_MAX_LENGTH: usize = get_formats_max_length(FORMATS, DEFAULT_FORMAT);

    let path_without_format_len = req_path.len() - MAX_RES_SEGMENT.len();

    let mut path = String::with_capacity(path_without_format_len + FORMAT_MAX_LENGTH);
    path.push_str(&req_path[..path_without_format_len]);

    for format in FORMATS {
        path.push_str(format);

        let mut url = proxy_url.clone();
        url.set_path(&path);

        if let Ok(res) = CLIENT
            .execute(create_request(req, req.method().clone(), url))
            .await
        {
            if res.status() == 200 {
                return Ok(res);
            }
        }

        path.truncate(path_without_format_len);
    }

    path.push_str(DEFAULT_FORMAT);
    proxy_url.set_path(&path);

    CLIENT
        .execute(create_request(req, req.method().clone(), proxy_url))
        .await
}

const fn get_formats_max_length(formats: &[&str], default_format: &str) -> usize {
    let mut max_len = default_format.len();
    let mut i = 0;
    let size = formats.len();

    while i < size {
        let len = formats[i].len();

        if len > max_len {
            max_len = len;
        }

        i += 1;
    }

    max_len
}

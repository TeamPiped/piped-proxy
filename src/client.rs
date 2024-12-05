use actix_web::HttpRequest;
use once_cell::sync::Lazy;
use reqwest::{Client, Method, Request, Url};
use std::env;
use std::net::{IpAddr, Ipv4Addr};

use crate::headers::is_header_allowed;

pub static CLIENT: Lazy<Client> = Lazy::new(|| {
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

    if crate::utils::get_env_bool("IPV4_ONLY") {
        builder.local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
    } else {
        builder
    }
    .build()
    .unwrap()
});

pub fn create_request(req: &HttpRequest, method: Method, url: Url) -> Request {
    let mut request = Request::new(method, url);
    let request_headers = request.headers_mut();

    for (key, value) in req.headers() {
        if is_header_allowed(key.as_str()) {
            request_headers.insert(key, value.clone());
        }
    }

    request
}

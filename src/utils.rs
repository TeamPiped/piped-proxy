use qstring::QString;
use reqwest::Url;
use std::borrow::Cow;
use std::collections::BTreeMap;

pub fn read_buf(buf: &[u8], pos: &mut usize) -> u8 {
    let byte = buf[*pos];
    *pos += 1;
    byte
}

fn finalize_url(path: &str, query: BTreeMap<String, String>) -> String {
    #[cfg(feature = "qhash")]
    {
        use std::collections::BTreeSet;
        use std::env;

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

pub fn localize_url(url: &str, host: &str) -> String {
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

pub fn escape_xml(raw: &str) -> Cow<'_, str> {
    if !raw.contains(&['<', '>', '&', '\'', '"'][..]) {
        // If there are no characters to escape, return the original string.
        Cow::Borrowed(raw)
    } else {
        // If there are characters to escape, build a new string with the replacements.
        let mut escaped = String::with_capacity(raw.len());
        for c in raw.chars() {
            match c {
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '&' => escaped.push_str("&amp;"),
                '\'' => escaped.push_str("&apos;"),
                '"' => escaped.push_str("&quot;"),
                _ => escaped.push(c),
            }
        }
        Cow::Owned(escaped)
    }
}

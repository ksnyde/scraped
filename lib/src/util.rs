use reqwest::header::HeaderValue;
use serde::Serialize;
use std::collections::HashMap;
use url::Url;

#[derive(Debug)]
pub struct BearerTokens {
    /// a bearer token that should be used for ALL URL's
    pub global: Option<HeaderValue>,
    /// a bearer token that should be used only for specified domains
    pub scoped: HashMap<String, HeaderValue>,
}

impl BearerTokens {
    pub fn new() -> BearerTokens {
        BearerTokens {
            global: None,
            scoped: HashMap::new(),
        }
    }

    /// get the bearer token for a given URL
    pub fn get(&self, url: Url) -> Option<HeaderValue> {
        if let Some(domain) = url.domain() {
            let token = self.scoped.get(domain);

            match token {
                Some(token) => Some(token.clone()),
                None => self.global.as_ref().cloned(),
            }
        } else {
            None
        }
    }
}

pub fn url_to_string<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    url.to_string().serialize(serializer)
}

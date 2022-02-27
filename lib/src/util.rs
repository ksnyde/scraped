use reqwest::{header::HeaderMap, Method};
use scraper::Html;
use serde::Serialize;
use url::Url;

use crate::document::PropertyCallback;

pub fn url_to_string<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    url.to_string().serialize(serializer)
}

pub fn html_to_string<S>(html: &Html, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    html.root_element().html().serialize(serializer)
}

pub fn method_to_string<S>(url: &Method, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    url.to_string().serialize(serializer)
}

pub fn headers_to_string<S>(headers: &HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    format!("{:?}", headers).serialize(serializer)
}

pub fn callback_to_string<S>(cb: &PropertyCallback, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    "propCallback()".serialize(serializer)
}

pub fn url_list_to_string<S>(url_list: &Vec<Url>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    url_list
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .serialize(serializer)
}

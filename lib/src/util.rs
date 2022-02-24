use std::collections::HashMap;

use serde::Serialize;
use serde_json::{json, Value};
use url::Url;

pub fn url_to_string<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    url.to_string().serialize(serializer)
}

pub fn serialize_selection<S>(
    selection: &Box<HashMap<String, Value>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    json!(selection).serialize(serializer)
}

pub fn serialize_selection_list<S>(
    list: &Vec<HashMap<String, Value>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    list.iter()
        .map(|v| json!(v))
        .collect::<Vec<Value>>()
        .serialize(serializer)
}

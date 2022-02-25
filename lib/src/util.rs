use serde::Serialize;
use url::Url;

pub fn url_to_string<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    url.to_string().serialize(serializer)
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

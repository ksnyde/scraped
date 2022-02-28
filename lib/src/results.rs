use color_eyre::{eyre::eyre, Result};
use reqwest::header::HeaderMap;
use scraper::Html;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
};
use tracing::trace;
use url::Url;

use crate::{
    document::{parse_url, LoadedDocument, PropertyCallback},
    selection::get_selection,
};

/// A recursive structure which provides the `url` and all top level
/// selectors on a given page as `data` and then optionally recurses
/// into child elements and provides the same structure.
#[derive(Debug, Serialize, Clone)]
pub struct ScrapedResults {
    /// The URL which was parsed.
    #[serde(serialize_with = "crate::util::url_to_string")]
    pub url: Url,
    /// The headers received in the page's response
    #[serde(serialize_with = "crate::util::headers_to_string")]
    pub headers: HeaderMap,
    /// The DOM nodes from the body of the page
    #[serde(serialize_with = "crate::util::html_to_string")]
    pub body: Html,
    /// the property values which were configured by passed in callbacks
    pub properties: HashMap<String, Value>,
    /// the selector results after applying the page's DOM tree
    pub selections: HashMap<String, Value>,
}

impl Display for ScrapedResults {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", serde_json::to_string(&self))
    }
}

impl From<&LoadedDocument<'_>> for ScrapedResults {
    /// converts a `LoadedDocument` and it's configuration into a fully
    /// parsed `ScrapedResults` struct
    fn from(doc: &LoadedDocument) -> ScrapedResults {
        trace!(
            "[{}]: converting LoadedDocument to ScrapedResults",
            &doc.url
        );
        let mut selections: HashMap<String, Value> = HashMap::new();
        let mut properties: HashMap<String, Value> = HashMap::new();

        doc.item_selectors.iter().for_each(|(k, sel)| {
            let el = doc.body.select(sel).next();
            if let Some(el) = el {
                let value = get_selection(el, doc.url);
                selections.insert(k.to_string(), json!(value));
            } else {
                selections.insert(k.to_string(), json!(null));
            }
        });
        doc.list_selectors.iter().for_each(|(k, sel)| {
            let value: Vec<HashMap<String, Value>> = doc //
                .body
                .select(sel)
                .map(|el| get_selection(el, doc.url))
                .collect();
            selections.insert(k.to_string(), json!(value));
        });
        trace!(
            "[{}]: {} selectors have been converted to values",
            &doc.url,
            &selections.keys().len()
        );

        doc.prop_callbacks.iter().for_each(|(k, cb)| {
            let cb: PropertyCallback = cb.extract();
            properties.insert(k.to_string(), cb(&selections));
        });

        ScrapedResults {
            url: doc.url.clone(),
            headers: doc.headers.clone(),
            body: doc.body.clone(),
            properties,
            selections: if *doc.keep_selectors {
                selections
            } else {
                HashMap::new()
            },
        }
    }
}

impl ScrapedResults {
    /// Returns a list of URL's which represent "child URLs". A child
    /// URL is determined by those _selectors_ which were deemed eligible
    /// when:
    /// 1. it is included in a call to `child_selectors(["foo", "bar"], scope)`
    /// 2. has a `href` property defined
    /// 3. the "scope" of the href first that defined in call to `child_selectors`
    pub fn get_child_urls(&self) -> Vec<Url> {
        let mut children: Vec<Url> = Vec::new();
        trace!("[{}]: getting the child URLs from page", self.url);

        for selector in self.selections.values() {
            match selector {
                // a list of selections
                Value::Array(list) => {
                    list.iter().for_each(|i| {
                        if let Value::Object(record) = i {
                            if let Some(Value::String(href)) = record.get("full_href") {
                                let url = parse_url(href);
                                children.push(url.unwrap());
                            }
                        }
                    });
                }
                // a singular/item selection
                Value::Object(obj) => {
                    if let Some(Value::String(href)) = obj.get("full_href") {
                        let url = parse_url(href).unwrap();
                        children.push(url);
                    }
                }
                _ => {}
            }
        }
        trace!(
            "got all child pages [{}] for \"{}\"",
            children.len(),
            self.url
        );

        children
    }

    /// Provides a convience accessor that allows selection of a particular
    /// _selection_ value or a _property_. If property and selection properties
    /// overlap then the property will mask the selection by the same name.
    ///
    /// If a non-existant key is passed in, this function will return an error.
    pub fn get(&self, key: &str) -> Result<Value> {
        if let Some(value) = self.properties.get(key) {
            Ok(json!(value))
        } else if let Some(value) = self.selections.get(key) {
            Ok(json!(value))
        } else {
            Err(eyre!(format!(
                "Request for the '{}' key in scraped results failed. This key does not exist!",
                key
            )))
        }
    }
}

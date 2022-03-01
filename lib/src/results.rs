use color_eyre::{eyre::eyre, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use scraper::Html;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
};
use tracing::trace;
use url::Url;

use crate::{
    document::{LoadedDocument, PropertyCallback},
    element::Element,
    selection::get_selection,
};

fn headers_to_hashmap(headers: &HeaderMap<HeaderValue>) -> HashMap<String, Vec<String>> {
    let mut header_hashmap = HashMap::new();
    for (k, v) in headers {
        let k = k.as_str().to_owned();
        let v = String::from_utf8_lossy(v.as_bytes()).into_owned();
        header_hashmap.entry(k).or_insert_with(Vec::new).push(v)
    }
    header_hashmap
}

/// A `SelectorNode` is the expected structure of a
/// _configured selector_. The two variants of this enum
/// map directly to whether the selector was chosen as
/// "list" selector or not.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SelectionResult {
    None(),
    /// A item selector results in this type of node
    Element(Element),
    /// A list selector results in this type of node
    List(Vec<Element>),
}

/// A recursive structure which provides the `url` and all top level
/// selectors on a given page as `data` and then optionally recurses
/// into child elements and provides the same structure.
#[derive(Debug, Serialize)]
pub struct ScrapedResults {
    /// The URL which was parsed.
    #[serde(serialize_with = "crate::util::url_to_string")]
    pub url: Url,
    /// The headers received in the page's response
    pub headers: HashMap<String, Vec<String>>,
    pub child_urls: Option<Vec<String>>,
    /// The DOM nodes from the body of the page
    #[serde(skip)]
    pub body: Html,
    /// the property values which were configured by passed in callbacks
    pub properties: HashMap<String, Value>,
    /// the selector results after applying the page's DOM tree
    pub selections: HashMap<String, SelectionResult>,
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
        let mut selections: HashMap<String, SelectionResult> = HashMap::new();
        let mut properties: HashMap<String, Value> = HashMap::new();

        doc.item_selectors.iter().for_each(|(k, sel)| {
            let el = doc.body.select(sel).next();
            if let Some(el) = el {
                let value = get_selection(el, doc.url);
                selections.insert(k.to_string(), SelectionResult::Element(value));
            } else {
                selections.insert(k.to_string(), SelectionResult::None());
            }
        });
        doc.list_selectors.iter().for_each(|(k, sel)| {
            let value: Vec<Element> = doc //
                .body
                .select(sel)
                .map(|el| get_selection(el, doc.url))
                .collect();
            if value.is_empty() {
                selections.insert(k.to_string(), SelectionResult::None());
            } else {
                selections.insert(k.to_string(), SelectionResult::List(value));
            }
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

        let mut result = ScrapedResults {
            url: doc.url.clone(),
            headers: headers_to_hashmap(&doc.headers),
            body: doc.body.clone(),
            properties,
            child_urls: None,
            selections: if *doc.keep_selectors {
                selections
            } else {
                HashMap::new()
            },
        };

        // get the child URLs if there are selectors to use
        if !doc.child_selectors.is_empty() {
            let selectors = doc.child_selectors;
            let urls = result.get_child_urls(selectors);
            result.child_urls = Some(urls);
        }

        result
    }
}

impl ScrapedResults {
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

    /// Returns a list of URL's which represent "child URLs". A child
    /// URL is determined by those _selectors_ which were deemed eligible
    /// when:
    /// 1. it is included in a call to `child_selectors(["foo", "bar"], scope)`
    /// 2. has a `href` property defined
    /// 3. the "scope" of the href first that defined in call to `child_selectors`
    fn get_child_urls(&self, selectors: &[String]) -> Vec<String> {
        let mut children: Vec<String> = Vec::new();
        trace!("[{}]: getting the child URLs from page", self.url);

        for (name, result) in &self.selections {
            if selectors.contains(name) {
                match result {
                    // a list of selections
                    SelectionResult::List(list) => {
                        list.iter().for_each(|i| {
                            if let Some(href) = &i.href {
                                children.push(href.to_string());
                            }
                        });
                    }
                    // a singular/item selection
                    SelectionResult::Element(obj) => {
                        if let Some(href) = &obj.href {
                            children.push(href.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        trace!(
            "got all child pages [{}] for \"{}\"",
            children.len(),
            self.url
        );

        children
    }
}

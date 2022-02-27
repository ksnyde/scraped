use color_eyre::{
    eyre::eyre,
    eyre::{Report, WrapErr},
    Result,
};

use core::fmt;
use reqwest::{header::HeaderMap, Method};
use scraper::{Html, Selector};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug, fmt::Display};
use url::Url;

use crate::results::ScrapedResults;

/// receives an unvalidated String and returns a validated Url
pub fn parse_url(url: &str) -> Result<Url, Report> {
    Url::parse(url)
        .map_err(|e| eyre!(e))
        .context(format!("Failed to parse the URL string recieved: {}", url))
}

/// a callback function which is provided a hashmap of all resultant _selectors_
/// and is expected to turn that into a meaningup JSON-based result.
pub type PropertyCallback = fn(sel: &HashMap<String, Value>) -> Value;

pub struct DebuggableCallback {
    text: &'static str,
    value: PropertyCallback,
}

impl Debug for DebuggableCallback {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl DebuggableCallback {
    /// wrap a callback function in way that allows debug functionality
    /// to be persisted
    pub fn new(cb: PropertyCallback) -> DebuggableCallback {
        DebuggableCallback {
            text: "callback function",
            value: cb,
        }
    }
    /// extract the callback function from the wrapper
    pub fn extract(&self) -> PropertyCallback {
        self.value
    }
}

#[derive(Debug)]
pub struct Document {
    /// The URL where the html document can be found
    pub url: Url,
    item_selectors: HashMap<String, Selector>,
    list_selectors: HashMap<String, Selector>,
    prop_callbacks: HashMap<String, DebuggableCallback>,
    /// the selectors which -- when including an href in their result -- are deemed to be child pages
    child_selectors: Vec<String>,
    /// indicates whether selector results will be
    /// kept in the result props
    keep_selectors: bool,
    method: Method,
}

impl Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Document[ {} {} ]: selectors: , ; props: ",
            self.method.as_str(),
            self.url.to_string()
        )
    }
}

impl From<&Url> for Document {
    fn from(url: &Url) -> Self {
        Document {
            url: url.clone(),
            keep_selectors: true,
            method: Method::GET,
            item_selectors: HashMap::new(),
            list_selectors: HashMap::new(),
            prop_callbacks: HashMap::new(),
            child_selectors: vec![],
        }
    }
}

impl Document {
    /// Returns a new Document; errors if string passed in is not a
    /// value URL
    pub fn new(url: &str) -> Result<Document> {
        Ok(Document {
            url: parse_url(url)?,
            keep_selectors: true,
            method: Method::GET,
            item_selectors: HashMap::new(),
            list_selectors: HashMap::new(),
            prop_callbacks: HashMap::new(),
            child_selectors: vec![],
        })
    }

    /// Add a selector for an item where the expectation is there is only one
    /// (or more specifically _at most_ one)
    pub fn add_selector(mut self, name: &str, selector: &str) -> Result<Self> {
        if let Ok(sel) = Selector::parse(selector) {
            self.item_selectors.insert(name.to_string(), sel);
            Ok(self)
        } else {
            Err(eyre!(format!(
                "'{}' is an invalid selector for the page: {}",
                selector, self.url
            )))
        }
    }

    /// Add a selector which is expect to bring a _list_ of results
    pub fn add_list_selector(mut self, name: &str, selector: &str) -> Result<Self> {
        if let Ok(sel) = Selector::parse(selector) {
            self.list_selectors.insert(name.to_string(), sel);
            Ok(self)
        } else {
            Err(eyre!(format!(
                "'{}' is an invalid selector for the page: {}",
                selector, self.url
            )))
        }
    }

    /// Adds some useful but generic selectors which includes:
    ///
    /// - `h1` through `h3`
    /// - `title`
    /// - `images`
    /// - `links`
    /// - `scripts`
    /// - `styles`
    /// - `meta`
    pub fn add_generic_selectors(mut self) -> Result<Self> {
        self.add_selector("h1", "h1")?
            .add_selector("title", "title")?
            .add_list_selector("h2", "h2")?
            .add_list_selector("h3", "h3")?
            .add_list_selector("links", "[href]")?
            .add_list_selector("images", "img")?
            .add_list_selector("scripts", "script")?
            .add_list_selector("styles", "[rel=\'stylesheet\']")?
            .add_list_selector("meta", "meta")?;

        Ok(self)
    }

    /// Parses into a `ParsedDoc` and then adds selectors intended to suit the `docs.rs` site.
    pub fn for_docs_rs(self) -> Result<Self> {
        self.add_selector("h1", "h1 .in-band a")?
            .add_selector("description", ".docblock")?
            .add_list_selector("h2", "h2")?
            .add_list_selector("modules", ".module-item a.mod")?
            .add_list_selector("structs", ".module-item a.struct")?
            .add_list_selector("functions", ".module-item a.fn")?
            .add_list_selector("traits", ".module-item a.trait")?
            .add_list_selector("enums", ".module-item a.enum")?
            .add_list_selector("macros", ".module-item a.macro")?
            .add_list_selector("type_defs", ".module-item a.type")?
            .add_list_selector("attr_macros", ".module-item a.attr")?
            .add_selector("desc", "section .docblock")?
            .child_selectors(vec![
                "modules",
                "structs",
                "functions",
                "traits",
                "types_defs",
                "enums",
                "macros",
            ])?;

        Ok(self)
    }

    /// States which _selectors_ -- where when an href is found on parsing -- are considered to
    /// be _child pages_ of the current document.
    ///
    /// **Note:** selectors must be set before you call this method or the selector will be _unknown_
    /// and return an error.
    pub fn child_selectors(mut self, selectors: Vec<&str>) -> Result<Self> {
        let mut invalid: Vec<&str> = vec![];

        // validate selectors
        &selectors.iter().for_each(|i| {
            if let None = self.item_selectors.get(*i) {
                if let None = self.list_selectors.get(*i) {
                    invalid.push(i)
                }
            }
        });

        if invalid.is_empty() {
            &selectors
                .into_iter()
                .for_each(|sel| self.child_selectors.push(sel.to_string()));
            Ok(self)
        } else {
            Err(
                eyre!(format!("When setting child selectors, references to invalid selectors where made! {:?} are invalid/unknown selectors.", invalid))
            )
        }
    }

    /// Add a **property** callback to the document's configuration.
    pub fn add_property(&self, name: &str, callback: PropertyCallback) -> Self {
        self.prop_callbacks
            .insert(name.to_string(), DebuggableCallback::new(callback));

        *self
    }

    /// Returns the scraped results by performing the following operations:
    ///
    /// 1. Requests the URL over the network
    /// 2. Parses the page's body with the `scrape` crate to get DOM representation
    /// 3. Applies the configured _selectors_ to the DOM to produce selector results
    /// 4. Uses _property_ callbacks to determine property results
    /// 5. Either returns
    pub async fn scrape(&self) -> Result<ScrapedResults> {
        todo!();
    }

    // /// Loads the HTTP page over the .
    // pub async fn load_document(&self) -> Result<LoadedDocument, Report> {
    //     let resp = match self.data {
    //         Some(v) => v,
    //         None => reqwest::get(self.url.to_string()).await?.text().await?,
    //     };

    //     Ok(LoadedDocument {
    //         url: self.url,
    //         data: resp,
    //     })
    // }

    /// if for some reason you want to provide the page's content yourself
    /// instead of having this crate load the page over the network you may
    /// do that.
    pub fn provide_response(headers: &HashMap<String, Value>, body: &str) {}
}

/// A document which has the web content loaded but not yet parsed into scraped results
#[derive(Debug)]
pub struct LoadedDocument {
    /// The URL where the html document can be found
    pub url: Url,
    /// the response headers returned by the page request
    pub headers: HeaderMap,
    /// The body of the message after having been parsed into
    /// a DOM structure by the `scrape` crate
    pub body: Html,
    /// DOM selectors expected to bring a singular item
    pub item_selectors: HashMap<String, Selector>,
    /// DOM selectors expected to bring a list of items
    pub list_selectors: HashMap<String, Selector>,
    /// callbacks used to build "properties" during parsing
    pub prop_callbacks: HashMap<String, DebuggableCallback>,
    /// the selectors which -- when including an href in their result -- are deemed to be child pages
    pub child_selectors: Vec<String>,
    /// indicates whether selector results will be
    /// kept in the result props
    pub keep_selectors: bool,
}

impl LoadedDocument {
    pub fn new(doc: Document, headers: HeaderMap, body: &str) -> LoadedDocument {
        LoadedDocument {
            url: doc.url,
            headers,
            body: Html::parse_document(body),
            item_selectors: doc.item_selectors,
            list_selectors: doc.list_selectors,
            prop_callbacks: doc.prop_callbacks,
            child_selectors: doc.child_selectors,
            keep_selectors: doc.keep_selectors,
        }
    }

    pub fn results(&self) {
        ScrapedResults::from(self);
    }
}

use color_eyre::{
    eyre::eyre,
    eyre::{Report, WrapErr},
    Help, Result,
};

use core::fmt;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT},
    Client, Response, StatusCode,
};
use scraper::{Html, Selector};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug, fmt::Display, future::Future};
use url::Url;

use crate::{
    results::{ScrapedResults, SelectionResult},
    util::BearerTokens,
};

/// receives an unvalidated String and returns a validated Url
pub fn parse_url(url: &str) -> Result<Url, Report> {
    Url::parse(url)
        .map_err(|e| eyre!(e))
        .context(format!("Failed to parse the URL string recieved: {}", url))
}

/// a callback function which is provided a hashmap of all resultant _selectors_
/// and is expected to turn that into a meaningup JSON-based result.
pub type PropertyCallback = fn(sel: &HashMap<String, SelectionResult>) -> Value;

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
    pub item_selectors: HashMap<String, Selector>,
    list_selectors: HashMap<String, Selector>,
    prop_callbacks: HashMap<String, DebuggableCallback>,
    /// the selectors which -- when including an href in their result -- are deemed to be child pages
    child_selectors: Vec<String>,
    /// indicates whether selector results will be
    /// kept in the result props
    keep_selectors: bool,
    req_headers: HeaderMap,
    /// bearer tokens sent in as configuration; scoped by URL
    bearer_tokens: BearerTokens,
}

impl Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Document[ {} ]", self.url)
    }
}

impl From<&Url> for Document {
    fn from(url: &Url) -> Self {
        Document {
            url: url.clone(),
            keep_selectors: true,
            item_selectors: HashMap::new(),
            list_selectors: HashMap::new(),
            prop_callbacks: HashMap::new(),
            child_selectors: vec![],
            req_headers: HeaderMap::new(),
            bearer_tokens: BearerTokens::new(),
        }
    }
}

impl Document {
    /// Returns a new Document; ParseError possible if invalid URL string
    pub fn new(url: &str) -> Result<Self> {
        let url = Url::parse(url).context(format!("Failed to parse the URL recieved: {}", url))?;
        Ok(Document {
            url,
            keep_selectors: true,
            item_selectors: HashMap::new(),
            list_selectors: HashMap::new(),
            prop_callbacks: HashMap::new(),
            child_selectors: vec![],
            req_headers: HeaderMap::new(),
            bearer_tokens: BearerTokens::new(),
        })
    }

    pub async fn build_request_client<F>(
        &self,
    ) -> impl Future<Output = Result<Response, reqwest::Error>>
    where
        F: Future<Output = Result<Response, reqwest::Error>>,
    {
        let client = Client::new();

        let headers = match self.bearer_tokens.get(self.url.clone()) {
            Some(token) => {
                let mut h = self.req_headers.clone();
                h.insert(AUTHORIZATION, token);
                h
            }
            None => self.req_headers.clone(),
        };

        let client = client.get(self.url.clone()).headers(headers).send();
        client
    }

    /// Add a selector for an item where the expectation is there is only one
    /// (or more specifically _at most_ one)
    pub fn add_selector<'a>(&'a mut self, name: &str, selector: &str) -> Result<&'a mut Document> {
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
    pub fn add_list_selector<'a>(
        &'a mut self,
        name: &str,
        selector: &str,
    ) -> Result<&'a mut Document> {
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
    pub fn add_generic_selectors(&'_ mut self) -> Result<&'_ mut Document> {
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
    pub fn for_docs_rs(&'_ mut self) -> Result<&'_ mut Document> {
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
                "type_defs",
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
    pub fn child_selectors<'a>(&'a mut self, selectors: Vec<&str>) -> Result<&'a mut Document> {
        let mut invalid: Vec<&str> = vec![];

        // validate selectors
        selectors.iter().for_each(|i| {
            if (self.item_selectors.get(*i).is_none()) && (self.list_selectors.get(*i).is_none()) {
                invalid.push(i)
            }
        });

        if invalid.is_empty() {
            selectors
                .into_iter()
                .for_each(|sel| self.child_selectors.push(sel.to_string()));
            Ok(self)
        } else {
            Err(
                eyre!(format!("When setting child selectors, references to invalid selectors where made! {:#?} are invalid/unknown selectors. Valid selectors were: {:#?} and {:#?}", invalid, self.item_selectors, self.list_selectors))
            )
        }
    }

    /// Add a **property** callback to the document's configuration.
    pub fn add_property<'a>(
        &'a mut self,
        name: &str,
        callback: PropertyCallback,
    ) -> &'a mut Document {
        self.prop_callbacks
            .insert(name.to_string(), DebuggableCallback::new(callback));

        self
    }

    /// Allows adding a bearer token for auth and/or rate-limiting purposes.
    ///
    /// Note: this token can be for ALL pages or scoped to a particular base URL.
    /// To scope it to a particular URL then you must separate/delimit the URL and
    /// token with the "|" character:
    ///
    /// ```rust
    /// use scraped::document::Document;
    /// let doc = Document::new("https://github.com")
    ///     .unwrap()
    ///     .bearer_token("github.com|{token}");
    /// ```
    pub fn bearer_token<'a>(&'a mut self, token: &'a str) -> Result<&'a mut Document> {
        if let Some((url, token)) = token.split_once('|') {
            let value: Result<HeaderValue> = format!("Bearer {}", token)
                .parse()
                .wrap_err("invalid bearer token");
            if let Ok(value) = value {
                self.bearer_tokens.scoped.insert(url.to_string(), value);
            } else {
                return Err(eyre!(format!("invalid bearer token: {}", token)));
            }
        } else {
            let value: Result<HeaderValue> = format!("Bearer {}", token)
                .parse()
                .wrap_err("invalid bearer token");
            if let Ok(value) = value {
                self.bearer_tokens.global = Some(value);
            } else {
                return Err(eyre!(format!("invalid bearer token: {}", token)));
            }
        }

        Ok(self)
    }

    /// Allows explicit setting of the user-agent string
    pub fn user_agent<'a>(&'a mut self, user_agent: &str) -> Result<&'a mut Document> {
        let user_agent = user_agent
            .parse()
            .wrap_err("The user_agent string was invalid")?;

        self.req_headers.insert(USER_AGENT, user_agent);

        Ok(self)
    }

    /// Returns the scraped results by performing the following operations:
    ///
    /// 1. Requests the URL over the network [[async]]
    /// 2. Parses the page's body with the `scrape` crate to get DOM representation
    /// 3. Applies the configured _selectors_ to the DOM to produce selector results
    /// 4. Uses _property_ callbacks to determine property results
    /// 5. Returns the `ScrapedResults` struct
    pub async fn scrape(&self) -> Result<ScrapedResults> {
        let client = Client::new();

        let headers = match self.bearer_tokens.get(self.url.clone()) {
            Some(token) => {
                let mut h = self.req_headers.clone();
                h.insert(AUTHORIZATION, token);
                h
            }
            None => self.req_headers.clone(),
        };

        let res = client
            .get(self.url.clone())
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                if e.status() == Some(StatusCode::TOO_MANY_REQUESTS) {
                    eyre!(e).section(format!("Rate limited while scraping: {}", self.url))
                } else {
                    eyre!(e).section(format!("Problem occurred while scraping: {}", self.url))
                }
            })?;

        let headers = res.headers().clone();
        let content = res.text().await?;
        let loaded = LoadedDocument::new(self, headers, &content);
        Ok(loaded.results())
    }

    /// if for some reason you want to provide the page's content yourself
    /// instead of having this crate load the page over the network you may
    /// do that.
    pub fn provide_response<'a>(&'a self, headers: HeaderMap, body: &str) -> LoadedDocument<'a> {
        LoadedDocument::new(self, headers, body)
    }
}

/// A document which has the web content loaded but not yet parsed into scraped results
#[derive(Debug)]
pub struct LoadedDocument<'a> {
    /// The URL where the html document can be found
    pub url: &'a Url,
    /// the _response_ headers returned by the page request
    pub headers: HeaderMap,
    /// The body of the message after having been parsed into
    /// a DOM structure by the `scrape` crate
    pub body: Html,
    /// DOM selectors expected to bring a singular item
    pub item_selectors: &'a HashMap<String, Selector>,
    /// DOM selectors expected to bring a list of items
    pub list_selectors: &'a HashMap<String, Selector>,
    /// callbacks used to build "properties" during parsing
    pub prop_callbacks: &'a HashMap<String, DebuggableCallback>,
    /// the selectors which -- when including an href in their result -- are deemed to be child pages
    pub child_selectors: &'a Vec<String>,
    /// indicates whether selector results will be
    /// kept in the result props
    pub keep_selectors: &'a bool,
}

impl<'a> LoadedDocument<'a> {
    pub fn new(doc: &'a Document, headers: HeaderMap, body: &str) -> LoadedDocument<'a> {
        LoadedDocument {
            url: &doc.url,
            headers,
            body: Html::parse_document(body),
            item_selectors: &doc.item_selectors,
            list_selectors: &doc.list_selectors,
            prop_callbacks: &doc.prop_callbacks,
            child_selectors: &doc.child_selectors,
            keep_selectors: &doc.keep_selectors,
        }
    }

    pub fn results(&self) -> ScrapedResults {
        ScrapedResults::from(self)
    }
}

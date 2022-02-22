use color_eyre::{
    eyre::eyre,
    eyre::{Report, WrapErr},
    Result,
};
use lazy_static::lazy_static;
use regex::Regex;
use results::{ParseResults, ResultKind};
use scraper::{Html, Selector};
use selection::{get_selection, SelectorKind};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use tokio_stream::StreamExt;
use tracing::{info, instrument, trace, warn};
use url::Url;

mod elements;
pub mod results;
pub mod selection;
mod util;

/// receives an unvalidated String and returns a validated Url
fn parse_url(url: &str) -> Result<Url, Report> {
    Url::parse(url)
        .map_err(|e| eyre!(e))
        .context(format!("Failed to parse the URL string recieved: {}", url))
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Document {
    /// The URL where the html document can be found
    #[serde(serialize_with = "util::url_to_string")]
    pub url: Url,
    pub data: Option<String>,
}

impl From<&Url> for Document {
    fn from(url: &Url) -> Self {
        Document {
            url: url.clone(),
            data: None,
        }
    }
}

impl Document {
    pub fn new(url: &str) -> Result<Document> {
        Ok(Document {
            url: parse_url(url)?,
            data: None,
        })
    }

    /// Loads the HTTP page over the network and saves as a string
    /// awaiting further processing.
    pub async fn load_document(self) -> Result<LoadedDocument, Report> {
        let resp = match self.data {
            Some(v) => v,
            None => reqwest::get(&self.url.to_string()).await?.text().await?,
        };

        Ok(LoadedDocument {
            url: self.url,
            data: resp,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadedDocument {
    #[serde(serialize_with = "util::url_to_string")]
    /// The URL where the html document can be found
    pub url: Url,
    /// the raw string data recieved via **Reqwest**
    pub data: String,
}

impl LoadedDocument {
    pub fn new(url: &str, data: &str) -> Result<LoadedDocument> {
        Ok(LoadedDocument {
            url: parse_url(url)?,
            data: data.to_string(),
        })
    }

    /// parses a `LoadedDocument` into a `ParsedDoc`
    pub fn parse_document(&self) -> Result<ParsedDoc> {
        ParsedDoc::new(self)
    }

    /// Parses into a `ParsedDoc` and then adds selectors intended to suit the `docs.rs` site.
    pub fn for_docs_rs(self) -> ParsedDoc {
        ParsedDoc::from(self)
            .add_selector("h1", "h1 .in-band a")
            .add_selector_all("h2", "h2")
            .add_selector_all("modules", ".module-item a.mod")
            .add_selector_all("structs", ".module-item a.struct")
            .add_selector_all("functions", ".module-item a.fn")
            .add_selector_all("traits", ".module-item a.trait")
            .add_selector_all("enums", ".module-item a.enum")
            .add_selector_all("macros", ".module-item a.macro")
            .add_selector_all("type_defs", ".module-item a.type")
            .add_selector_all("attr_macros", ".module-item a.attr")
            .add_selector("desc", "section .docblock")
            .child_selectors(
                vec![
                    "modules",
                    "structs",
                    "functions",
                    "traits",
                    "types_defs",
                    "enums",
                    "macros",
                ],
                ChildScope::Relative(),
            )
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum ChildScope {
    /// any child selector with a `href` property will be included
    All(),
    /// only child selectors with _relative_ path in their `href` property will be included
    Relative(),
    /// only child selectors with an _absolute_ path in their `href` property will be included
    Absolute(),
    /// both relative and absolute links to an HTTP resource allowed, other ref types
    /// (e.g., file, Javascript calls, ...) are excluded
    Http(),
    File(),
}

/// validates that the scoping rules allow the href value and
/// returns Some(url) if in scope.
///
/// Note: in the case of a "relative path", this function will
/// modify this to be a fully qualified path
fn validate_child_href(href: &str, scope: &ChildScope, current_page: &Url) -> Option<Url> {
    lazy_static! {
        static ref REL: Regex = Regex::new(r"^[\w\.#]+$").unwrap();
    }

    let url = match (
        href,
        scope,
        href.starts_with("http"),
        href.starts_with("file"),
        REL.captures(href).is_some(),
    ) {
        (_, ChildScope::All(), false, false, true) => {
            Some([&current_page.to_string(), href].join("/"))
        }
        (_, ChildScope::All(), _, _, _) => Some(href.to_string()),
        (_, ChildScope::Http(), true, _, _) => Some(href.to_string()),
        (_, ChildScope::Http(), false, _, _) => None,
        (_, ChildScope::File(), _, true, _) => Some(href.to_string()),
        (_, ChildScope::File(), _, false, _) => None,
        (_, ChildScope::Relative(), _, _, true) => {
            Some([&current_page.to_string(), href].join("/"))
        }

        _ => None,
    }?;

    match parse_url(&url) {
        Ok(url) => Some(url),
        Err(_e) => {
            // on error just log the problem to console and return None to skip child
            None
        }
    }
}

/// A `Document` which has been loaded from the network and parsed
/// into a DOM tree. You can add "selectors" which will be lazily
/// evaluated when calling `get(selector)` or when exporting as
/// a JSON payload.
pub struct ParsedDoc {
    pub url: Url,
    pub html: Html,
    /// a hash of selectors which will be lazily evaluated when
    /// converting to a JSON output or when calling `get(selector)`
    /// to extract a particular selector.
    pub selectors: HashMap<String, SelectorKind>,
    /// allows user to build up a set of selectors which will be looked
    /// as being candidates for selecting
    child_selectors: Vec<(String, ChildScope)>,
    /// a dictionary of user defined callbacks which leverage the
    /// selectors as input to produce clean outcomes
    properties: HashMap<String, fn(sel: &HashMap<String, ResultKind>) -> Value>,
}

impl ParsedDoc {
    #[instrument]
    pub fn new(doc: &LoadedDocument) -> Result<ParsedDoc> {
        Ok(ParsedDoc {
            url: doc.url.clone(),
            html: Html::parse_document(&doc.data),
            selectors: HashMap::new(),
            child_selectors: vec![],
            properties: HashMap::new(),
        })
    }

    /// Adds some useful but generic selectors which includes:
    ///
    /// - `title`
    /// - `images`
    /// - `links`
    /// - `scripts`
    /// - `styles`
    /// - `meta`
    pub fn add_generic_selectors(self) -> Self {
        self.add_selector_all("links", "[href]")
            .add_selector("title", "title")
            .add_selector_all("images", "img")
            .add_selector_all("scripts", "script")
            .add_selector_all("styles", "[rel=\'stylesheet\']")
            .add_selector_all("meta", "meta")
    }

    /// Add a selector for an item where the expectation is there is only one
    /// (or more specifically _at most_ one)
    pub fn add_selector(mut self, name: &str, selector: &str) -> Self {
        let selector = Selector::parse(selector).unwrap();
        self.selectors
            .insert(name.to_string(), SelectorKind::Item(selector));

        self
    }

    /// Add a selector which is expect to bring a _list_ of results
    pub fn add_selector_all(mut self, name: &str, selector: &str) -> Self {
        let selector = Selector::parse(selector).unwrap();
        self.selectors
            .insert(name.to_string(), SelectorKind::List(selector));

        self
    }

    /// allows for the expression of which selectors are intended to point to a
    /// "child page" of the current page. Those designated selectors which have
    /// an `href` property as well as the correct "scope" will be scraped as well
    /// when the CLI's `--follow` flag is set or when the `results_graph()` function
    /// is called.
    pub fn child_selectors(mut self, selectors: Vec<&str>, scope: ChildScope) -> Self {
        let new_selectors: Vec<(String, ChildScope)> = selectors
            .iter()
            .map(|s| ((*s).to_string(), scope.clone()))
            .collect();

        new_selectors
            .iter()
            .for_each(|s| self.child_selectors.push(s.clone()));

        self
    }

    /// add a property callback to the configuration
    pub fn add_property(
        mut self,
        name: &str,
        callback: fn(sel: &HashMap<String, ResultKind>) -> Value,
    ) -> Self {
        self.properties.insert(name.to_string(), callback);

        self
    }

    /// Gets the results of a _specific_ selector or property.
    ///
    /// **Note:** if a property of the same name of a selector exists then the
    /// the property will be given precedence; effectively masking the selector value
    pub fn get(&self, name: &str) -> Result<Option<ResultKind>, Report> {
        match (self.selectors.get(name), self.properties.get(name)) {
            (_, Some(cb)) => {
                trace!("getting {} as a property", name);
                let selectors = self.get_selection_results();
                let result = cb(&selectors);
                Ok(Some(ResultKind::Property(result)))
            }
            (Some(SelectorKind::Item(v)), _) => {
                if let Some(el) = self.html.select(&v).next() {
                    trace!("getting {} as a SelectorKind::Item", name);
                    Ok(Some(ResultKind::Item(Box::new(get_selection(
                        el, &self.url,
                    )))))
                } else {
                    Ok(None)
                }
            }
            (Some(SelectorKind::List(v)), _) => {
                trace!("getting {} as a SelectorKind::List", name);

                Ok(Some(ResultKind::List(
                    self.html
                        .select(&v)
                        .map(|el| get_selection(el, &self.url))
                        .collect(),
                )))
            }
            _ => return Err(eyre!("could not find the '{}' selector", name.to_string())),
        }
    }

    /// Returns a list of URL's which represent "child URLs". A child
    /// URL is determined by those _selectors_ which were deemed eligible
    /// when:
    /// 1. it is included in a call to `child_selectors(["foo", "bar"], scope)`
    /// 2. has a `href` property defined
    /// 3. the "scope" of the href first that defined in call to `child_selectors`
    pub fn get_child_urls(&self) -> Vec<Url> {
        let mut children = Vec::new();
        trace!("getting the child URLs for {}", self.url);

        for (name, selector) in &self.selectors {
            if let Some((_, scope)) = self //
                .child_selectors
                .iter()
                .find(|(s, _)| s == name)
            {
                match selector {
                    SelectorKind::List(v) => {
                        // iterate through all elements
                        self.html.select(v).for_each(|c| {
                            if let Some(href) = get_selection(c, &self.url).href {
                                if let Some(href) = validate_child_href(&href, scope, &self.url) {
                                    children.push(href);
                                }
                            }
                        });
                    }
                    SelectorKind::Item(v) => {
                        if let Some(el) = self.html.select(v).next() {
                            // if selector returned an element, get href prop (if avail)
                            if let Some(href) = get_selection(el, &self.url).href {
                                if let Some(v) = validate_child_href(&href, scope, &self.url) {
                                    children.push(v)
                                }
                            }
                        }
                    }
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

    /// Streams in the child HTML pages and parses them into `ParsedDoc`
    /// structs.
    pub async fn get_children(&self) -> Result<Vec<ParseResults>, Report> {
        let urls = self.get_child_urls();
        trace!(
            "retrieving {} child URLs for {} over network",
            urls.len(),
            self.url
        );
        let mut children: Vec<ParseResults> = vec![];
        let mut stream = tokio_stream::iter(urls);

        while let Some(v) = stream.next().await {
            let doc = Document::from(&v);
            let child = doc.load_document().await.unwrap().for_docs_rs();
            trace!("getting {}", &child.url);
            children.push(child.results());
            info!("finished loading {}", &child.url);
        }

        Ok(children)
    }

    /// applies all _selector configuration_ on the current page content to arrive at
    /// selection _results_.
    pub fn get_selection_results(&self) -> HashMap<String, ResultKind> {
        let mut data: HashMap<String, ResultKind> = HashMap::new();

        self.selectors
            .iter()
            .for_each(|(name, _)| match self.get(name) {
                Ok(Some(v)) => {
                    data.insert(name.to_string(), v);
                }
                _ => {
                    eyre!("Problem inserting the results for the selector '{}'.", name,);
                }
            });

        data
    }

    /// provides the _selector results to all property callbacks and returns a hashmap of
    /// _property values_.
    pub fn get_property_results(&self) -> Result<HashMap<String, Value>> {
        let selectors = self.get_selection_results();
        let mut results: HashMap<String, Value> = HashMap::new();
        self.properties.keys().for_each(|k| {
            let cb = self.properties.get(k);
            if let Some(cb) = cb {
                results.insert(k.to_string(), cb(&selectors));
            }
        });

        Ok(results)
    }

    /// Returns all _selectors_ and _properties_ on the current page without recursing
    /// into child pages.
    pub fn results(&self) -> ParseResults {
        let data = self.get_selection_results();
        let props = self
            .get_property_results()
            .expect("properties were not ready");

        ParseResults {
            url: self.url.clone(),
            data,
            props,
            children: vec![],
        }
    }

    /// Returns a tree of `ParseResults` starting with the given URL and
    /// then following into the children nodes (one level deep).
    pub async fn results_graph(&self) -> Result<ParseResults, Report> {
        let mut current_page = self.results();
        current_page.children = self.get_children().await?;

        Ok(current_page)
    }
}

impl From<LoadedDocument> for ParsedDoc {
    fn from(doc: LoadedDocument) -> Self {
        ParsedDoc {
            url: doc.url,
            html: Html::parse_document(&doc.data),
            selectors: HashMap::new(),
            child_selectors: vec![],
            properties: HashMap::new(),
        }
    }
}

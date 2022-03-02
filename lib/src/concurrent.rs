use futures_util::{
    stream::{self, Iter},
    Stream, StreamExt,
};
use reqwest::{Client, Response};
use serde::Serialize;
use std::{collections::HashMap, str::Bytes};
use tracing::info;
use url::Url;
const CONCURRENT_REQUESTS: usize = 2;

use crate::{document::Document, results::ScrapedResults, util::parse_urls};
#[derive(Debug, Serialize)]
pub enum Buffering {
    None,
    Unordered,
    /// buffering for a stream which will guarentee the order of futures inserted into
    /// it is preserved. This type of buffering needs a "buffering factor" to be configured.
    Ordered(usize),
}

#[derive(Serialize)]
pub struct ConcurrentScrape {
    /// You can configure documents to be scraped as just
    /// URL's and it will then use the globally configured
    /// selectors/properties as it converts these URL's into
    /// Documents.
    #[serde(skip)]
    pub urls: Vec<Url>,
    /// All documents passed in will be loaded, parsed,
    #[serde(skip)]
    pub docs: Vec<Document>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub results: HashMap<String, ScrapedResults>,
    pub config: ScrapeConfig,
}

async fn get_page(
    client: &Client,
    url: &Url,
) -> Result<(Response, impl Stream<Item = Result<Bytes, reqwest::Error>>), reqwest::Error> {
    match client.get(url.clone()).send().await {
        // if we're able to connect, then we're ready to start streaming the body
        Ok(resp) => {
            // return response (for header and url info), and then a stream for body of page
            (resp, stream::iter(resp.bytes().await))
        }
        Err(e) => Err(e),
    }
}

async fn get_pages(
    pages: &Vec<Url>,
) -> impl Stream<Item = (Response, impl Stream<Item = Result<Bytes, reqwest::Error>>)> {
    let client = Client::new();
    stream::iter(pages).then(|i| get_page(&client, i))
}

#[derive(Serialize)]
pub struct ScrapeConfig {
    delay: (usize, usize),
    concurrency: usize,
    buffering: Buffering,
}

impl ConcurrentScrape {
    pub fn new() -> Self {
        ConcurrentScrape {
            urls: vec![],
            docs: vec![],
            results: HashMap::new(),
            config: ScrapeConfig {
                delay: (30, 10),
                concurrency: 2,
                buffering: Buffering::Unordered,
            },
        }
    }

    pub fn add_urls(&self, urls: Vec<&str>) {
        let url_results = parse_urls(urls);
        if !url_results.failures.is_empty() {
            eprintln!(
                "{} urls failed to be parsed: {:?}",
                url_results.failures.len(),
                url_results.failures
            );
        }

        if !url_results.urls.is_empty() {
            url_results.urls.into_iter().for_each(|u| self.urls.push(u));
        } else {
            eprintln!("All URLs passed in were unable to be parsed; none added for scraping!");
        }
    }

    // pub fn add_docs(&'a self, docs: Vec<&'a Document>) {
    //     // TODO
    // }

    /// Set a baseline delay between requests, with a variant amount of randomness
    /// (or 0 for no randomness)
    // pub fn set_delay(&self, delay: usize, randomness: usize) -> Self {
    //     self.config.delay = (delay, randomness);

    //     *self
    // }

    pub async fn execute_old(self) {
        // let from_docs: Vec<&Url> = self.docs.iter().map(|d| &d.url).collect();
        let client = Client::new();
        let mut urls = self.urls.clone();
        self.docs.into_iter().for_each(|d| urls.push(d.url.clone()));

        info!("starting concurrent requests for urls: {:?}", urls);

        let requests = stream::iter(urls)
            .map(|url| {
                let client = &client;
                async move {
                    let resp = client.get(url.clone()).send().await?;
                    println!("requesting page at {}", &url);
                    info!("requesting page at {}", &url);
                    resp.bytes().await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS)
            .for_each(|b: Result<Bytes, reqwest::Error>| async {
                match b {
                    Ok(b) => {
                        println!("had {} bytes", b.len());
                        let s = format!("{:?}", b.slice(..));
                        println!("slice: {}", s);
                    }
                    // outcome.push((Url::parse("https://ken.net").unwrap(), HeaderMap::new(), {
                    //     let s = b.all().collect().to_string();

                    //     s
                    // }));
                    // }
                    Err(e) => {
                        println!("Got an error: {:?}", e);
                    }
                }
            })
            .await;

        println!("{:?}", requests);
        // Box::new(successes)
    }

    /// Runs through all provided URLs and provides ParsedResults structs from them.
    /// In order to do this it must:
    ///
    /// - concurrently make requests for all URLs (both `url` property and URLs from `docs` property)
    ///     - to preserve ability to match string result to the URL, we must use an ordered buffering
    ///     -
    pub async fn execute(self) {
        // established ordered URL list and reusable client
        let client = Client::new();
        let mut urls = self.urls.clone();
        self.docs.into_iter().for_each(|d| urls.push(d.url.clone()));

        let pages = get_pages(&urls).await;
    }
}

pub enum ByteStreamOutcome {
    Success(String),
    Error(reqwest::Error),
}

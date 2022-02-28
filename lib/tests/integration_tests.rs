use claim::{assert_err, assert_ok, assert_some};
use color_eyre::{eyre::eyre, Result};
use reqwest::header::HeaderMap;
use scraped::{
    document::{Document, LoadedDocument},
    results::ScrapedResults,
};
use serde_json::{json, Value};
use std::fs;
use url::Url;

fn load_simple_doc<'a>(doc: &'a Document) -> LoadedDocument<'a> {
    let headers = HeaderMap::new();
    let body =
        fs::read_to_string("tests/fixtures/simple-doc.html").expect("Problem reading fixture file");
    doc.provide_response(headers, &body)
}

#[test]
fn valid_string_url_is_accepted() {
    let url = String::from("https://dev.null");

    assert_ok!(Document::new(&url));
}

#[test]
fn invalid_string_url_is_rejected() {
    let url = String::from("\\x!//");
    assert_err!(Document::new(&url));
}

#[test]
fn document_from_url() {
    let str = "https://google.com";
    let url = Url::parse(str).unwrap();
    let from_url = Document::from(&url);
    let from_str = Document::new(str).unwrap();
    assert_eq!(from_url.url, from_str.url);
}

#[test]
fn single_selector_matches() -> Result<()> {
    let mut doc = Document::new("https://dev.null").unwrap();
    doc.add_selector("h1", "h1").unwrap();

    let result = ScrapedResults::from(&load_simple_doc(&doc));
    let h1 = result.get("h1")?;

    if let Value::Object(h1) = h1 {
        let text = h1.get("text");
        if text.is_none() {
            Err(eyre!("h1's text property was none"))
        } else {
            if let Value::String(text) = text.unwrap() {
                if text.eq("My Title") {
                    Ok(())
                } else {
                    Err(eyre!(format!(
                        "the h1 text value was supposed to be 'My Title' not {}",
                        text
                    )))
                }
            } else {
                Err(eyre!("h1's text property wasn't a string type"))
            }
        }
    } else {
        Err(eyre!("foobar"))
    }
}

#[test]
fn property_definition_available_in_results() {
    let mut doc = Document::new("https://dev.null").unwrap();
    doc.add_property("hello", |_| json!("world"));
    let results = ScrapedResults::from(&load_simple_doc(&doc));

    assert!(results.get("hello").is_ok());
    let hello = results.get("hello").unwrap();
    assert_eq!(&hello, &json!("world"));
}

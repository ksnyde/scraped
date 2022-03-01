use claim::{assert_err, assert_ok};
use color_eyre::{eyre::eyre, Result};
use reqwest::header::HeaderMap;
use scraped::{
    document::{Document, LoadedDocument},
    results::ScrapedResults,
};
use serde_json::{json, Value};
use std::fs;
use url::Url;

/// loads any file in fixtures directory
fn load_fixture<P: AsRef<Path>>(path: P) -> String {
    let path: Path = "tests/fixtures/".join(path);
    fs::read_to_string(path).expect(format!("Problem reading fixture file: {}", path))
}

/// given a `Document` and HTML fixture, converts to a `LoadedDocument`
fn load_document<'a, P: AsRef<Path>>(doc: &'a Document, fixture: P) -> LoadedDocument<'a> {
    let body = load_fixture(fixture);
    doc.provide_response(HeaderMap::new(), &body)
}

fn load_simple_doc<'a>(doc: &'a Document) -> LoadedDocument<'a> {
    load_document(doc, "simple-doc.html")
    // let headers = HeaderMap::new();
    // let body =
    //     fs::read_to_string("tests/fixtures/simple-doc.html").expect("Problem reading fixture file");
    // doc.provide_response(headers, &body)
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
fn using_rust_selectors_on_simple_html_works_but_no_result_returned() -> Result<()> {
    let mut doc = Document::new("https://dev.null").unwrap();
    doc.for_docs_rs().unwrap();

    let result = ScrapedResults::from(&load_simple_doc(&doc));
    let valid_but_empty = result.get("structs");
    assert_ok!(&valid_but_empty);
    assert_eq!(valid_but_empty.unwrap(), Value::Null);

    Ok(())
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
        Err(eyre!("h1 was not an object"))
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

#[test]
fn serialized_results_avoid_empty_props() {
    let mut doc = Document::new("https://dev.null").unwrap();
    doc.add_selector("h1", "h1").unwrap();

    let result = ScrapedResults::from(&load_simple_doc(&doc));

    dbg!(result);
}

#[test]
fn serilized_results_show_non_explicit_props() {
    todo!();
}

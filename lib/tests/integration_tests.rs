use std::fs;

use claim::{assert_err, assert_ok, assert_some};
use scraped::{Document, LoadedDocument};
use serde_json::json;
use url::Url;

#[test]
fn valid_string_url_is_accepted() {
    let url = String::from("https://dev.null");

    assert_ok!(Document::new(&url));
    assert_ok!(LoadedDocument::new(&url, &"".to_string()));
}

#[test]
fn invalid_string_url_is_rejected() {
    let url = String::from("\\x!//");
    assert_err!(Document::new(&url));
    assert_err!(LoadedDocument::new(&url, &"".to_string()));
}

#[test]
fn document_from_url() {
    let url = Url::parse("https://google.com").unwrap();
    assert_eq!(Document::from(&url), Document { url, data: None });
}

#[test]
fn single_selector_matches() {
    let url = String::from("https://dev.null");
    let contents =
        fs::read_to_string("tests/fixtures/simple-doc.html").expect("Problem reading fixture file");
    let doc = LoadedDocument::new(&url, &contents)
        .expect("LoadedDoc prepped")
        .parse_document()
        .expect("LoadedDoc parsed")
        .add_selector("h1", "h1");
    let result = doc.get("h1");
    assert_ok!(&result);
    let result = result.unwrap();
    assert_some!(&result);
    let _result = result.expect("result has a value");
    // assert_eq!(result, "My Title")
}

#[test]
fn static_property_definition_available_in_results() {
    let url = String::from("https://dev.null");
    let contents =
        fs::read_to_string("tests/fixtures/simple-doc.html").expect("Problem reading fixture file");
    let results = LoadedDocument::new(&url, &contents)
        .expect("LoadedDoc created")
        .parse_document()
        .expect("ParsedDoc created")
        .add_property("hello", |_| json!("world"))
        .results();

    assert_some!(results.props.get("hello"));
    let hello = results.props.get("hello").expect("hello prop exists");
    assert_eq!(hello, &json!("world"));
}

// fn single_selector_without_match() {
//     //
// }

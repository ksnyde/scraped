use std::fs;

use claim::{assert_err, assert_ok};
use scraped::{Document, LoadedDocument};
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

fn single_selector_matches() {
    let url = String::from("https://dev.null");
    let contents =
        fs::read_to_string("tests/fixtures/simple-doc.html").expect("Problem reading fixture file");
    let doc = LoadedDocument::new(&url, &contents)?
        .parse_document()?
        .add_generic_selectors();
    let result = doc.get("h1");
    assert_eq()
}

// fn single_selector_without_match() {
//     //
// }

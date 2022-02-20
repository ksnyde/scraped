use claim::{assert_err, assert_ok};
use scraped::{Document, LoadedDocument};

#[test]
fn valid_url_is_accepted() {
    let url = String::from("https://dev.null");

    assert_ok!(Document::new(&url));
    assert_ok!(LoadedDocument::new(&url, "".to_string()));
}

#[test]
fn invalid_url_is_rejected() {
    let url = String::from("\\x!//");
    assert_err!(Document::new(&url));
    assert_err!(LoadedDocument::new(&url, "".to_string()));
}

// fn single_selector_matches() {
//     let url = String::from("https://dev.null");
//     let contents =
//         fs::read_to_string("tests/fixtures/simple-").expect("Problem reading fixture file");
//     let doc = LoadedDocument::new(&url, contents);
// }

// fn single_selector_without_match() {
//     //
// }

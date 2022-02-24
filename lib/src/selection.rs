use scraper::{ElementRef, Selector};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::trace;
use url::Url;

/// Gets all the name/value pairs available on the passed in DOM
/// node and returns as a serde::Value enumeration
pub fn get_selection(el: ElementRef, url: &Url) -> HashMap<String, Value> {
    let mut selection: HashMap<String, Value> = HashMap::new();
    let props = el.value().attrs();
    let tag = el.value().name().to_string();

    // add the DOM's tag name
    selection.insert(String::from("tagName"), json!(tag));

    // extract all key/value pairs in DOM node
    props.into_iter().for_each(|(k, v)| {
        selection.insert(k.to_string(), Value::String(v.to_string()));
    });
    trace!(
        "[{}] got key/value props on a DOM node: {:?}",
        url,
        selection
    );

    // text and html
    let text = el.text().collect::<String>();
    if text.len() > 0 {
        selection.insert(String::from("text"), json!(text.trim().to_string()));
    }
    if !el.inner_html().is_empty() {
        selection.insert(
            String::from("html"),
            json!(el.inner_html().trim().to_string()),
        );
    }

    // synthesize other props conditionally

    let image_formats = ["gif", "jpg", "jpeg", "avif", "webp", "ico", "tiff"];
    // let doc_formats = ["doc", "docx", "pdf"];
    // let data_formats = ["csv", "json", "txt", "xls"];
    // let code_formats = ["js", "wasm"];
    // let style_formats = ["css", "scss", "sass"];

    if let Some(Value::String(href)) = selection.get("href") {
        if href.starts_with("http") {
            let href = href.to_string();
            selection.insert(String::from("hrefType"), json!("absolute"));
            selection.insert(String::from("full_href"), json!(href));
        } else {
            let href = href.to_string();
            let url = url.join(&href).expect("full url should be parsable");
            if href.len() == 0 {
                selection.insert(String::from("hrefType"), json!("empty"));
            } else {
                selection.insert(String::from("hrefType"), json!("relative"));
                selection.insert(String::from("full_href"), json!(format!("{}", url)));
            }
        }
        trace!("[{}] added a full_href prop", url);
    }

    if let Some(Value::String(src)) = selection.get("src") {
        let ext = src.split(".").last();
        // if we can see a file extension
        if let Some(ext) = ext {
            let is_image_format = image_formats.contains(&ext);

            if (tag == "img".to_string()) || (is_image_format) {
                if let Some(v) = image_formats.into_iter().find(|i| (*i) == ext) {
                    selection.insert(String::from("imageType"), json!(v.to_string()));
                }
                selection.insert(String::from("sourceType"), json!("image"));
            }
        }
    }

    trace!("selection for node completed");

    selection
}

#[derive(Debug)]
pub enum SelectorKind {
    /** a selector with a single DOM element as result */
    Item(Selector),
    /** a selector with a _list_ of DOM elements as a result */
    List(Selector),
}

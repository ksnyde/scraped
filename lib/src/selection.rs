use crate::elements;
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::trace;
use url::Url;

/// A `Selection` captures the key characteristics of a part of the DOM tree that
/// is intersection of an HTML document (`Html`) and a selector (`Selector`).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Selection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// The `href` property, if present on the selected element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    /// the fully qualified URL for the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_href: Option<String>,

    /// The `src` property, if present on the selected element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// The plain text within the selected DOM element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The innerHTML of the selected DOM element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,

    /// While not a heavily used prop in the body of HTML it is very
    /// common to have this -- paired with "name" -- in the meta properties
    /// of a page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    /// other -- less used props -- can still be stored
    /// but they will be stored as a JSON hash value in
    /// this `other` property to avoid too many props.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, Value>,
}

impl Selection {
    fn new() -> Self {
        Selection {
            id: None,
            class: None,
            style: None,
            name: None,
            href: None,
            full_href: None,
            text: None,
            html: None,
            content: None,
            rel: None,
            src: None,
            type_: None,
            disabled: None,

            other: HashMap::new(),
        }
    }
}

pub fn get_selection(el: ElementRef, url: &Url) -> Selection {
    let mut selection = Selection::new();

    selection.id = elements::id(&el);
    selection.class = elements::class(&el);
    selection.style = elements::style(&el);
    selection.text = elements::text(&el);
    selection.html = elements::html(&el);
    selection.href = [elements::href(&el), elements::href_only_child(&el)]
        .into_iter()
        .flatten()
        .next();
    selection.full_href = match &selection.href {
        Some(href) => {
            if href.starts_with("http") {
                Some(href.to_string())
            } else {
                let url = url.join(href).expect("full url should be parsable");

                Some(format!("{}", url))
            }
        }
        _ => None,
    };

    selection.name = elements::name(&el);
    selection.content = elements::content(&el);
    selection.rel = elements::rel(&el);
    selection.src = elements::src(&el);
    selection.type_ = elements::type_(&el);
    selection.disabled = elements::disabled(&el);

    trace!(
        "[{:?}] selection completed: {:?}, {:?}",
        url.to_string(),
        selection.full_href,
        selection.text,
    );

    selection
}

#[derive(Debug)]
pub enum SelectorKind {
    /** a selector with a single DOM element as result */
    Item(Selector),
    /** a selector with a _list_ of DOM elements as a result */
    List(Selector),
}

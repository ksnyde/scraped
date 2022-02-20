use crate::elements;
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The `href` property, if present on the selected element
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The `src` property, if present on the selected element
    pub src: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The plain text within the selected DOM element
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The innerHTML of the selected DOM element
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

impl From<ElementRef<'_>> for Selection {
    fn from(el: ElementRef) -> Self {
        let mut selection = Selection::new();
        selection.id = elements::id(&el);
        selection.class = elements::class(&el);
        selection.style = elements::style(&el);
        selection.text = elements::text(&el);
        selection.html = elements::html(&el);
        selection.href = elements::href(&el);
        selection.name = elements::name(&el);
        selection.content = elements::content(&el);
        selection.rel = elements::rel(&el);
        selection.src = elements::src(&el);
        selection.type_ = elements::type_(&el);

        selection
    }
}

#[derive(Debug)]
pub enum SelectorKind {
    /** a selector with a single DOM element as result */
    Item(Selector),
    /** a selector with a _list_ of DOM elements as a result */
    List(Selector),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum SelectionKind {
    /** a selector with a single DOM element as result */
    Item(Box<Selection>),
    /** a selector with a _list_ of DOM elements as a result */
    List(Vec<Selection>),
}

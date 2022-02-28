use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::instrument;

#[derive(Deserialize)]
pub enum ImageFormat {
    Gif(),
    Jpeg(),
    Avif(),
    Webp(),
    Ico(),
    Tiff(),
    Png(),
    Svg(),
    Other(String),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceType {
    Image(),
    Code(),
    Style(),
    Doc(),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HrefType {
    Relative(),
    /// A fully qualified name was used to notate
    Absolute(),
    /// the href property called a page level Javascript function
    Javascript(),
    /// there was a href property but it's value was empty
    Empty(),
    /// a type of relative link which just offsets the current page
    /// to a particular location. H2 and H3 tags often use this as a
    /// way to move to the right part of a longer page.
    AnchorLink(),
    /// it isn't that uncommon to have an AnchorLink but
    /// with a value just "#" which doesn't link to a particular
    /// part of the page
    SelfReferencingAnchor(),
}

/// an `Element` represents the DOM node which was targetted
/// by the document's selector.
///
/// In the case of a _list_ selector we will h
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    /// the plain text contained within the element
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    /// the `innerHtml()` of the element
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,

    /// the tag name (aka, "a", "h1", "button", etc.) of the element
    tag_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_type: Option<SourceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_type: Option<ImageFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    full_href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    href_type: Option<HrefType>,

    #[serde(rename(deserialize = "type"))]
    _type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rel: Option<String>,
}

/// A `SelectorNode` is the expected structure of a
/// _configured selector_. The two variants of this enum
/// map directly to whether the selector was chosen as
/// "list" selector or not.
#[derive(Deserialize)]
pub enum SelectorNode {
    Element(Element),
    List(Vec<Element>),
}

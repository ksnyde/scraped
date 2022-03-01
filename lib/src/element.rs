use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum ImageType {
    Gif,
    Jpeg,
    Avif,
    Webp,
    Ico,
    Tiff,
    Png,
    Svg,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetType {
    /// image source
    Image,
    /// code source (e.g., JS, WASM, etc.)
    Code,
    /// style source (typically CSS, possibly SCSS, etc.)
    Style,
    /// link to a document source (e.g., word doc, pdf, etc.)
    Doc,
    Data,
    /// another HTML page on the same site as currently scraped site
    HtmlSameSite,
    /// another HTML page but one which is on a different host than
    /// the current page being scraped
    HtmlForeignSite,

    Font,

    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HrefType {
    /// A relative path to a location on the same website
    Relative,
    /// A fully qualified name was used to notate
    Absolute,
    /// the href property called a page level Javascript function
    Javascript,
    /// there was a href property but it's value was empty
    Empty,
    /// a type of relative link which just offsets the current page
    /// to a particular location. H2 and H3 tags often use this as a
    /// way to move to the right part of a longer page.
    AnchorLink,
    /// it isn't that uncommon to have an AnchorLink but
    /// with a value just "#" which doesn't link to a particular
    /// part of the page
    SelfReferencingAnchor,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HrefSource {
    /// the element selected has the `href` attribute on it directly
    Element,
    /// the element does _not_ have an `href` but the inner HTML is composed of a
    /// singular element which _does_ have an HREF. This often happens with sections
    /// tagged as H2, H3, etc.
    OnlyChild,
    /// the element does _not_ have an `href` attribute but amongst the interior
    /// elements, there is a single element which does.
    SingularChild,
    /// The element does _not_ have an `href` attribute but multiple interior elements
    /// do. In this case the `href` property will _not_ be set but this flag indicates
    /// the potential for a more refined selector picking up a desired `href`.
    MultipleInterior,
}

/// an `Element` represents the DOM node which was targetted
/// by the document's selector.
///
/// In the case of a _list_ selector we will h
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    /// the tag name (aka, "a", "h1", "button", etc.) of the element
    pub tag_name: String,

    /// the plain text contained within the element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// the `innerHtml()` of the element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// identifies the broad category of target that this url
    /// appears to be pointing at (e.g., )
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<TargetType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_type: Option<ImageType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href_source: Option<HrefSource>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href_type: Option<HrefType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(deserialize = "type"))]
    pub _type: Option<String>,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub attrs: HashMap<String, Value>,
}

impl Element {
    pub fn new(tag_name: &str) -> Element {
        Element {
            tag_name: tag_name.to_string(),
            text: None,
            html: None,
            // id: None,
            // class: None,
            // style: None,
            src: None,
            target_type: None,
            image_type: None,

            href: None,
            full_href: None,
            href_type: None,
            href_source: None,

            _type: None,
            // name: None,
            // rel: None,
            attrs: HashMap::new(),
        }
    }
}

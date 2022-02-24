use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    vec,
};
use tracing::warn;
use url::Url;

use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum ResultKind {
    /** a selector with a single DOM element as result */
    #[serde(serialize_with = "crate::util::serialize_selection")]
    Item(Box<HashMap<String, Value>>),
    /** a selector with a _list_ of DOM elements as a result */
    #[serde(serialize_with = "crate::util::serialize_selection_list")]
    List(Vec<HashMap<String, Value>>),
    /** a property which has been synthesized from selection results */
    Property(Value),
}

impl Display for ResultKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            ResultKind::Property(_) => write!(f, "{}", &self),
            _ => write!(f, "{}", json!(&self).to_string()),
        }
    }
}

impl ResultKind {
    pub fn get(&self, key: &str) -> Value {
        match self {
            ResultKind::Item(value) => {
                if let Some(v) = value.get(key) {
                    json!((*v).clone())
                } else {
                    json!(null)
                }
            }
            ResultKind::List(v) => v //
                .into_iter()
                .map(|i| json!(i.get(key)))
                .collect(),
            ResultKind::Property(v) => match v {
                Value::Object(v) => {
                    json!(v)
                }
                _ => {
                    warn!("There was an attempt to get the key '{}' from a non-object property! This will result in a null value being returned.", key);
                    json!(null)
                }
            },
        }
    }
}

/// A recursive structure which provides the `url` and all top level
/// selectors on a given page as `data` and then optionally recurses
/// into child elements and provides the same structure.
#[derive(Debug, Serialize, Clone)]
pub struct ParseResults {
    /// The URL which was parsed.
    #[serde(serialize_with = "crate::util::url_to_string")]
    pub url: Url,
    /// The raw data extracted from the CSS selectors specified.
    pub data: HashMap<String, ResultKind>,
    /// Abstracted properties derived from `data` and converted to
    /// abstract JSON representation for serialization.s
    pub props: HashMap<String, Value>,

    pub children: Vec<ParseResults>,
}

impl Display for ParseResults {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", serde_json::to_string(&self))
    }
}

/// A singular "result" that is typically fit into a flat vector of results
#[derive(Clone, Serialize)]
pub struct FlatResult {
    /// The URL which was parsed.
    #[serde(serialize_with = "crate::util::url_to_string")]
    pub url: Url,
    /// The raw data extracted from the CSS selectors specified.
    pub data: HashMap<String, ResultKind>,
    /// Abstracted properties derived from `data` and converted to
    /// abstract JSON representation for serialization.s
    pub props: HashMap<String, Value>,
}

impl FlatResult {
    /// flattens a `ParseResults` struct from it's heirarchical structure to a
    /// vector of `FlatResult` results.
    pub fn flatten(r: &ParseResults) -> Vec<FlatResult> {
        let mut flat = vec![FlatResult {
            url: r.url.clone(),
            data: r.data.clone(),
            props: r.props.clone(),
        }];

        r.children.iter().for_each(|c| {
            FlatResult::flatten(c)
                .iter()
                .for_each(|i| flat.push(i.clone()));
        });

        flat
    }
}

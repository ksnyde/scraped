use color_eyre::{eyre::eyre, Result};
use lazy_static::__Deref;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};
use tracing::warn;
use url::Url;

use serde::Serialize;
use serde_json::{json, Value};

use crate::ParsedDoc;

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum ResultKind {
    /** a selector with a single DOM element as result */
    Item(Box<HashMap<String, Value>>),
    /** a selector with a _list_ of DOM elements as a result */
    List(Vec<HashMap<String, Value>>),
    /** a property which has been synthesized from selection results */
    Property(Value),
}

impl Display for ResultKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            ResultKind::Property(_) => write!(f, "{}", &self),
            _ => write!(f, "{}", json!(&self)),
        }
    }
}

impl ResultKind {
    pub fn get(&self, key: &str) -> Value {
        match self {
            ResultKind::Item(value) => {
                if let Some(v) = value.get(key) {
                    json!((*v))
                } else {
                    json!(null)
                }
            }
            ResultKind::List(v) => v //
                .iter()
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
pub struct ParsedResults {
    /// The URL which was parsed.
    #[serde(serialize_with = "crate::util::url_to_string")]
    pub url: Url,
    /// The raw data extracted from the CSS selectors specified.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub data: HashMap<String, ResultKind>,
    /// Abstracted properties derived from `data` and converted to
    /// abstract JSON representation for serialization.s
    pub props: HashMap<String, Value>,

    /// the URLs which were identified from selectors activated with
    /// child_selectors() method.
    #[serde(serialize_with = "crate::util::url_list_to_string")]
    pub child_urls: Vec<Url>,

    cache: Option<HashMap<String, Value>>,
}

impl Display for ParsedResults {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", serde_json::to_string(&self))
    }
}

impl From<&ParsedDoc> for ParsedResults {
    fn from(doc: &ParsedDoc) -> ParsedResults {
        ParsedResults {
            url: doc.url.clone(),
            data: doc.get_selection_results(),
            props: doc.get_property_results(),
            child_urls: doc.get_child_urls(),
            cache: None,
        }
    }
}

fn merge_from_ref(map: &mut HashMap<(), ()>, map_ref: &HashMap<(), ()>) {
    map.extend(map_ref.into_iter().map(|(k, v)| (k.clone(), v.clone())));
}

impl ParsedResults {
    /// provides a convience method that allows selection of from
    /// a HashMap which is the merge of all selectors (Item and List)
    /// with properties where properties will be given preference and
    /// will mask a similarly named selector
    ///
    /// Note: if a key is passed in that is NOT defined then an error
    /// will be raised, however if a valid key is used that relates to
    /// an empty/non-existant selector than you will get a `Value::Null`
    /// variant of `Value`.
    pub fn get(&self, key: &str) -> Result<Value> {
        let props = self.props.clone().extend(self.data);

        let value: Option<&Value> = match props {
            Some(cache) => cache.get(key),
            _ => {
                // build cache
                let mut cache = HashMap::new();
                for (k, v) in self.data.iter() {
                    cache.insert(k, v);
                }

                &cache.get(key)
            }
        };

        if let Some(_v) = value {
            Ok(value.unwrap().deref().clone())
        } else {
            Err(eyre!(
                "Couldn't find the property or selector called '{}'",
                key
            ))
        }
    }
}

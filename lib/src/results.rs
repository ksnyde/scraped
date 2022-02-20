use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use serde::Serialize;
use serde_json::Value;

use crate::selection::SelectionKind;

/// A recursive structure which provides the `url` and all top level
/// selectors on a given page as `data` and then optionally recurses
/// into child elements and provides the same structure.
#[derive(Debug, Serialize)]
pub struct ParseResults {
    /// The URL which was parsed.
    pub url: String,
    /// The raw data extracted from the CSS selectors specified.
    pub data: HashMap<String, SelectionKind>,
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

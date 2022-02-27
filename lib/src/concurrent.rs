use std::collections::HashMap;

// use futures_util::{stream, StreamExt};

use crate::document::Document;

pub struct ConcurrentScrape {
    docs: Vec<Document>,
    config: HashMap<String, String>,
}

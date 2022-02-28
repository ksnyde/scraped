use std::collections::HashMap;

// use futures_util::{stream, StreamExt};

use crate::document::Document;

pub struct ConcurrentScrape {
    pub docs: Vec<Document>,
    pub config: HashMap<String, String>,
}

impl ConcurrentScrape {
    pub fn new() -> Self {
        ConcurrentScrape {
            docs: vec![],
            config: HashMap::new(),
        }
    }
}

use color_eyre::eyre::Result;
use scraped::results::ParsedResults;
use serde_json::json;
use tracing::{info, trace, warn};

/// outputs a set of properties which reside on a `ParsedDoc`.
pub fn show(doc: &ParsedResults, show: &Option<String>) {
    let props = match show {
        Some(v) => v.split(',').into_iter().collect(),
        None => vec![],
    };
    trace!("showing properties: {:?}", props);

    props.into_iter().for_each(|k| {
        let v = doc.get(k);
        println!("- {}: {}", k, v);
    });
}

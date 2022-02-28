use scraped::results::ScrapedResults;
use tracing::{debug, instrument, warn};

/// outputs a set of properties which reside on a `ParsedDoc`.
#[instrument]
pub fn show(doc: &ScrapedResults, show: &Option<String>) {
    let props = match show {
        Some(v) => v.split(',').into_iter().collect(),
        None => vec![],
    };
    debug!("properties configured to show are: {:?}", props);

    props.into_iter().for_each(|k| {
        let result = doc.get(k);

        if let Ok(v) = result {
            println!("- {}: {}", k, v);
        } else {
            //
        }
    });
}

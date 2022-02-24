use color_eyre::eyre::Result;
use scraped::ParsedDoc;
use serde_json::json;
use tracing::{info, trace, warn};

/// outputs a set of properties which reside on a `ParsedDoc`.
pub fn show(doc: &ParsedDoc, show: &Option<String>) -> Result<()> {
    let props = match show {
        Some(v) => v.split(",").into_iter().collect(),
        None => vec![],
    };
    trace!("showing properties: {:?}", props);

    props.into_iter().for_each(|p| match doc.get(p) {
        Ok(value) => {
            if let Some(value) = value {
                trace!("value for '{}' was found: {}", p, value);
                println!("- {}: {}", p, json!(value));
            } else {
                info!("no value found for '{}'", p);
                println!("- {}: undefined", p);
            }
        }
        Err(_err) => {
            warn!("The property '{}' was not found", p);
            println!("- {}: not found!", p);
        }
    });

    Ok(())
}

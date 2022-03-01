use clap::Parser;
use color_eyre::Result;
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info};
// use config;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The URL to inspect
    url: String,

    #[clap(short, long, parse(from_os_str))]
    /// the file where JSON results will be saved
    output: Option<PathBuf>,

    #[clap(short, long)]
    /// Follow document into child links
    follow: bool,

    #[clap(long)]
    /// Flatten results to a JSON array of pages
    flatten: bool,

    #[clap(short, long)]
    /// Show a specific _selector_ as part of console output; use "all" to show all selectors and "props"
    /// to show only configured _properties_
    show: Option<String>,

    #[clap(short, long)]
    /// Pass in a JSON configuration file to add your own selectors and properties
    config: Option<PathBuf>,
}

use scraped::{
    concurrent::ConcurrentScrape,
    document::{Document, PropertyCallback},
    results::SelectionResult,
};
mod show;
use show::show;

#[tokio::main]
async fn main() -> Result<()> {
    // let format = tracing_subscriber::fmt::format().compact();
    tracing_subscriber::fmt::init();
    // LogTracer::init()?;
    color_eyre::install()?;

    let title: PropertyCallback = |r| {
        if let Some(SelectionResult::Element(title)) = r.get("title") {
            if let Some(SelectionResult::Element(h1)) = r.get("h1") {
                let choices: Vec<String> = [h1.text.clone(), title.text.clone()] //
                    .into_iter()
                    .filter_map(|i| match i.is_some() {
                        true => Some(i.unwrap()),
                        false => None,
                    })
                    .collect();

                return json!(choices.first());
            };
        };

        json!(null)
    };

    let args = Args::parse();
    debug!("CLI arguments parsed {:?}", args);

    let mut doc = Document::new(&args.url)?;
    doc //
        .add_generic_selectors()?
        .for_docs_rs()?
        .add_property("title", title);
    let results = doc.scrape().await?;

    println!("- Scraped {} ", &args.url);
    // log to console
    show(&results, &args.show);

    // process children
    let _children = ConcurrentScrape::new();
    if args.follow {
        // TODO
    }

    match (&args.output, args.follow) {
        (Some(v), false) => {
            let results = serde_json::to_string(&results)?;
            fs::write(&v, results).await?;
        }
        (Some(_v), true) => {
            // println!(
            //     "- Loading and parsing {} child nodes{}",
            //     &doc.get_child_urls().len(),
            //     if args.flatten { " [flatten] " } else { "" }
            // );

            // let results = match (args.follow, args.flatten) {
            //     (true, true) => {
            //         let r = FlatResult::flatten(&doc.results_graph().await?);
            //         serde_json::to_string(&r)?
            //     }
            //     (true, false) => serde_json::to_string(&doc.results_graph().await?)?,
            //     (false, _) => serde_json::to_string(&doc.results_graph().await?)?,
            // };

            // fs::write(&v, results).await?;
        }
        _ => (),
    }

    info!("completed CLI command");

    Ok(())
}

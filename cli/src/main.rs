use clap::Parser;
use color_eyre::Result;
use serde_json::{json, Value};
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
    document::{Document, PropertyCallback},
    results::ScrapedResults,
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
        let title = r
            .get("title")
            .expect("The title selector should exist")
            .get("text");
        let h1 = r //
            .get("h1")
            .expect("The title selector should exist")
            .get("text");

        let choices: Vec<&Value> = [&h1, &title]
            .into_iter()
            .filter(|i| !Value::is_null(*i))
            .collect();

        json!(choices.first())
    };

    let args = Args::parse();
    debug!("CLI arguments parsed {:?}", args);

    let doc = ScrapedResults::from(
        &Document::new(&args.url)?
            .load_document()
            .await?
            .for_docs_rs()
            .add_generic_selectors()
            .add_property("title", title),
    );

    let d = Document::new("https://foo.bar")?.load_document().await?;

    println!("- Parsed {} ", &args.url);

    show(&doc, &args.show);

    match (&args.output, args.follow) {
        (Some(v), false) => {
            let results = serde_json::to_string(&doc.results()?)?;
            fs::write(&v, results).await?;
        }
        (Some(_v), true) => {
            println!(
                "- Loading and parsing {} child nodes{}",
                &doc.get_child_urls().len(),
                if args.flatten { " [flatten] " } else { "" }
            );

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

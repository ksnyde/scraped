use clap::Parser;
use color_eyre::eyre::Result;
use std::path::PathBuf;
use tokio::fs;

#[derive(Parser)]
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

use scraped::Document;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let doc = Document::new(&args.url)?
        .load_document()
        .await?
        .for_docs_rs()
        .add_generic_selectors();
    println!("- Parsed {} ", &args.url);

    match (&args.output, args.follow) {
        (Some(v), false) => {
            let results = serde_json::to_string(&doc.results())?;
            fs::write(&v, results).await?;
        }
        (Some(v), true) => {
            println!(
                "- Loading and parsing {} child nodes",
                &doc.get_child_urls().len()
            );
            let results = serde_json::to_string(&doc.results_graph().await?)?;
            fs::write(&v, results).await?;
        }
        _ => (),
    }

    Ok(())
}

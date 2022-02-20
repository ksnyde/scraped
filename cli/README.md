# Scraped CLI

This repo leverages the popular `clasp` CLI builder to wrap the `scraped` crate as a native binary.

```bash
# help
scraped --help
# scrape a specific URL, showing all `h2` nodes
scraped https://docs.rs/crate/latest --show h2
# scrape with a specifically defined set of properties and output results to a file
scraped https://docs.rs/crate/latest --config settings.json -o output.json 
```


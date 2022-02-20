# Scraped

A Rust web scraper which provides a high-level configuration interface to scrape meta data from URL based resources.

This is a monorepo which consists of the following packages:

- `lib` - the rust "scraped" crate / library
- `cli` - the CLI is packaged in **npm** module system as "scraped" a NAPI-RS API providing native performance to the Windows, Linux, and macOS targets
- `wasm` - secondarily we package a CLI that compiles the core library down to a WASM target and wraps this with a thin Javascript wrapper so that it can easily work on any platform

## Backstory

This package was originally developed to help facilitate building search indexes for the Rust API documents of the [Tauri](https://tauri.studio) project and still supports that task but it has grown to provide a more generalized interface that can be used for lots of scraping needs.

> this library sits on the shoulder of the `scrape` crate which provides all the lower level DOM selector querying capabilities.
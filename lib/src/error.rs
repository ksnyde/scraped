use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum ScrapedError {
    #[error("the URL was not able to be parsed")]
    InvalidUrl(#[from] ParseError),
}

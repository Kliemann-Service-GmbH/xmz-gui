use reqwest;
use url;


#[derive(Debug)]
pub enum Error {
    BackendError,
    ReqwestError(reqwest::Error),
    UrlParseError(url::ParseError),
    XMZError(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::ReqwestError(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::UrlParseError(err)
    }
}

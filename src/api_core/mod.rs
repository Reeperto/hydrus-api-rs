use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

pub mod access_management;
pub mod adding_files;
pub mod adding_tags;
pub mod adding_urls;
pub mod client;
pub mod client_builder;
pub mod common;
pub mod managing_cookies_and_http_headers;
pub mod managing_pages;
pub mod searching_and_fetching_files;

pub use searching_and_fetching_files::file_sort_type;

pub(crate) trait Endpoint {
    type Request: Serialize + Debug;
    type Response: DeserializeOwned + Debug;

    fn path() -> String;
}

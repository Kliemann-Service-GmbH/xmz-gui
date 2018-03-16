#![deny(unused_extern_crates)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate reqwest;
extern crate url;

#[macro_use] mod utils;
mod model;
pub mod backend;
pub mod error;
pub mod globals;
pub mod types;

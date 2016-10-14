#![feature(custom_derive)]
#![feature(proc_macro)]
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod server;
pub mod analytics;


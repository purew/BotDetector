//! A proof-of-concept of a reverse-proxy detecting bots/scrapers.
//! 
//!
//! 
//! 
//! 
#![warn(missing_docs)]
#![feature(custom_derive)]
#![feature(proc_macro)]
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate hyper;
#[macro_use]
extern crate iron;

#[macro_use]
extern crate log;

pub mod server;
pub mod analytics;
pub mod detector;


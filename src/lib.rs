#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

//! # Fluent logger for Rust

pub mod logger;

pub mod sender;

extern crate chrono;

extern crate serde;

extern crate serde_json;

extern crate rmp_serde;

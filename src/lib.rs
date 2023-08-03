//! # dehashed-rs
//!
//! This crate provides a SDK for the API of [dehashed](https://dehashed.com/).
//!
//! Please note, that this project is not an official implementation from dehashed.
//!
//! ## Usage

#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![warn(missing_docs)]

pub use api::*;
pub use error::DehashedError;
#[cfg(feature = "tokio")]
pub use scheduler::*;

mod api;
mod error;
pub(crate) mod res;
#[cfg(feature = "tokio")]
mod scheduler;
#[cfg(test)]
mod tests;

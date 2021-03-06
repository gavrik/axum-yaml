//! YAML extractor for axum
//! 
//! This crate provides struct `Yaml` that can be used to extract typed information from request's body.
//! 
//! serde-yaml parser under the hood.
//! 
#[macro_use]
pub(crate) mod macros;

#[cfg(test)]
mod tests;

pub mod yaml;
pub mod rejection;

use axum::{BoxError, Error};

pub use crate::yaml::Yaml;
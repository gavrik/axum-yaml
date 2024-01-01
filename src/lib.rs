//! YAML extractor for axum
//! 
//! This crate provides struct `Yaml` that can be used to extract typed information from request's body.
//! 
//! [`serde-yaml`] parser under the hood.

mod macros;

#[cfg(test)]
mod test_client;

pub mod yaml;
pub mod rejection;

pub use crate::yaml::Yaml;

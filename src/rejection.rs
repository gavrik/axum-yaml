// We only use the pre-existing `BytesRejection` from `axum_core` because it does not qualify as a private API
use axum_core::extract::rejection::BytesRejection;

use crate::macros::{
    __composite_rejection as composite_rejection, __define_rejection as define_rejection,
};

define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Failed to deserialize the YAML body into the target type"]
    /// Rejection type for `Yaml` that takes the [`serde_yaml::Error`] type.
    ///
    /// This rejection is used when the request body cannot be deserialized
    /// into the target type or contains syntactically invalid YAML.
    pub struct YamlError(Error);
}

define_rejection! {
    #[status = UNSUPPORTED_MEDIA_TYPE]
    #[body = "Expected request with `Content-Type: application/yaml`"]
    /// Rejection type for `Yaml` used if the `Content-Type`
    /// header is missing.
    pub struct MissingYamlContentType;
}

composite_rejection! {
    pub enum YamlRejection {
        YamlError,
        MissingYamlContentType,
        BytesRejection,
    }
}

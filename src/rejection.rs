// We only use the pre-existing `BytesRejection` from `axum_core` because it does not qualify as a private API
use axum_core::extract::rejection::BytesRejection;

use crate::macros::{
    __define_rejection as define_rejection,
    __composite_rejection as composite_rejection,
};

define_rejection! {
    #[status = UNPROCESSABLE_ENTITY]
    #[body = "Failed to deserialize the YAML body into the target type"]
    /// Rejection type for `Yaml`.
    ///
    /// This rejection is used if the request body is syntactically valid YAML but couldn't be
    /// deserialized into the target type.
    pub struct YamlDataError(Error);
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
        YamlDataError,
        MissingYamlContentType,
        BytesRejection,
    }
}

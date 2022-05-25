pub use axum_core::extract::rejection::*;

//use crate::macros::{define_rejection};

pub use axum_core::extract::rejection::*;

//#[cfg(feature = "yaml")]
define_rejection! {
    #[status = UNPROCESSABLE_ENTITY]
    #[body = "Failed to deserialize the YAML body into the target type"]
    pub struct YamlDataError(Error);
}
/*
define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Failed to parse the request body as YAML"]
    pub struct YamlSyntaxError(Error);
}
*/
define_rejection! {
    #[status = UNSUPPORTED_MEDIA_TYPE]
    #[body = "Expected request with `Content-Type: application/yaml`"]
    pub struct MissingYamlContentType;
}

//#[cfg(feature = "yaml")]
composite_rejection! {
    pub enum YamlRejection {
        YamlDataError,
        //YamlSyntaxError,
        MissingYamlContentType,
        BytesRejection,
    }
}

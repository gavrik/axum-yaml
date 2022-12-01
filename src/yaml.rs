//! YAML extractor for axum
//! 
//! This crate provides struct `Yaml` that can be used to extract typed information from request's body.
//! 
//! serde-yaml parser under the hood.
//! 
use axum::{
    body::{Bytes, HttpBody},
    extract::FromRequest,
    BoxError, 
    async_trait,
};
use axum::response::{IntoResponse, Response};
use axum::http::{
    header::{self, HeaderValue},
    StatusCode,
    Request,
};
use serde::{de::DeserializeOwned, Serialize};
use std::ops::{Deref, DerefMut};

use crate::rejection::*;
/// YAML Extractor / Response.
///
/// When used as an extractor, it can deserialize request bodies into some type that
/// implements [`serde::Deserialize`]. If the request body cannot be parsed, or it does not contain
/// the `Content-Type: application/yaml` header, it will reject the request and return a
/// `400 Bad Request` response.
///
/// # Extractor example
///
/// ```rust,no_run
/// use axum::{
///     extract,
///     routing::post,
///     Router,
/// };
/// use serde::Deserialize;
/// use axum_yaml::Yaml;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     email: String,
///     password: String,
/// }
///
/// async fn create_user(Yaml(payload): Yaml<CreateUser>) {
///     // payload is a `CreateUser`
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
///
/// When used as a response, it can serialize any type that implements [`serde::Serialize`] to
/// `YAML`, and will automatically set `Content-Type: application/yaml` header.
///
/// # Response example
///
/// ```
/// use axum::{
///     extract::Path,
///     routing::get,
///     Router,
/// };
/// use serde::Serialize;
/// use uuid::Uuid;
/// use axum_yaml::Yaml;
///
/// #[derive(Serialize)]
/// struct User {
///     id: Uuid,
///     username: String,
/// }
///
/// async fn get_user(Path(user_id) : Path<Uuid>) -> Yaml<User> {
///     let user = find_user(user_id).await;
///     Yaml(user)
/// }
///
/// async fn find_user(user_id: Uuid) -> User {
///     // ...
///     # unimplemented!()
/// }
///
/// let app = Router::new().route("/users/:id", get(get_user));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Yaml<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for Yaml<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync + 'static,
{
    type Rejection = YamlRejection;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        if yaml_content_type(&req) {
            let bytes = Bytes::from_request(req, state).await?;

            let value = match serde_yaml::from_slice(&bytes) {
                Ok(value) => value,
                Err(err) => {
                    let rejection = YamlDataError::from_err(err).into();
                    return Err(rejection);
                }
            };
            Ok(Self(value))
        } else {
            Err(MissingYamlContentType.into())
        }

    }
}

fn yaml_content_type<B>(req: &Request<B>) -> bool {
    let content_type = if let Some(content_type) = req.headers().get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_yaml_content_type = mime.type_() == "application"
        && (mime.subtype() == "yaml");
    
    is_yaml_content_type
}

impl<T> Deref for Yaml<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Yaml<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Yaml<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for Yaml<T>
where 
    T: Serialize,
{
    fn into_response(self) -> Response {
        match serde_yaml::to_vec(&self.0) {
            Ok(bytes) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("application/yaml").unwrap(),
                )],
                bytes,
            ).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            ).into_response(),
        }
    }
}

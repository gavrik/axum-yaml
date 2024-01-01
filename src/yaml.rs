use std::ops::{Deref, DerefMut};

use async_trait::async_trait;
use axum_core::{
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use bytes::{BufMut, Bytes, BytesMut};
use http::{header, HeaderMap, HeaderValue, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

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
/// ```no_run
/// use axum::{
///     extract,
///     routing::post,
///     Router,
/// };
/// use axum_yaml::Yaml;
/// use serde::Deserialize;
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
/// #   axum::serve(
/// #       tokio::net::TcpListener::bind("").await.unwrap(),
/// #       app.into_make_service(),
/// #   )
/// #   .await
/// #   .unwrap();
/// # };
/// ```
///
/// When used as a response, it can serialize any type that implements [`serde::Serialize`] to
/// `YAML`, and will automatically set `Content-Type: application/yaml` header.
///
/// # Response example
///
/// ```no_run
/// use axum::{
///     extract::Path,
///     routing::get,
///     Router,
/// };
/// use axum_yaml::Yaml;
/// use serde::Serialize;
/// use uuid::Uuid;
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
/// #   axum::serve(
/// #       tokio::net::TcpListener::bind("").await.unwrap(),
/// #       app.into_make_service(),
/// #   )
/// #   .await
/// #   .unwrap();
/// # };
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Yaml<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Yaml<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = YamlRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if yaml_content_type(req.headers()) {
            let bytes = Bytes::from_request(req, state).await?;
            Self::from_bytes(&bytes)
        } else {
            Err(MissingYamlContentType.into())
        }
    }
}

fn yaml_content_type(headers: &HeaderMap) -> bool {
    let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
        return false;
    };

    let Ok(content_type) = content_type.to_str() else {
        return false;
    };

    let Ok(mime) = content_type.parse::<mime::Mime>() else {
        return false;
    };

    let is_yaml_content_type = mime.type_() == "application"
        && (mime.subtype() == "yaml" || mime.suffix().map_or(false, |name| name == "yaml"));

    is_yaml_content_type
}

impl<T> Deref for Yaml<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Yaml<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Yaml<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> Yaml<T>
where
    T: DeserializeOwned,
{
    /// Construct a `Yaml<T>` from a byte slice. Most users should prefer to use the `FromRequest` impl
    /// but special cases may require first extracting a `Request` into `Bytes` then optionally
    /// constructing a `Yaml<T>`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, YamlRejection> {
        let deserializer = serde_yaml::Deserializer::from_slice(bytes);

        match serde_path_to_error::deserialize(deserializer) {
            Ok(value) => Ok(Yaml(value)),
            Err(err) => Err(YamlError::from_err(err).into()),
        }
    }
}

impl<T> IntoResponse for Yaml<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        match serde_yaml::to_writer(&mut buf, &self.0) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/yaml"),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::routing::post;
    use axum::Router;
    use http::StatusCode;
    use serde::Deserialize;
    use serde_yaml::Value;

    use crate::test_client::TestClient;

    #[tokio::test]
    async fn deserialize_body() {
        #[derive(Debug, Deserialize)]
        struct Input {
            foo: String,
        }

        let app = Router::new().route("/", post(|input: Yaml<Input>| async { input.0.foo }));

        let client = TestClient::new(app);
        let res = client
            .post("/")
            .body("foo: bar")
            .header("content-type", "application/yaml")
            .await;

        let body = res.text().await;
        assert_eq!(body, "bar");
    }

    #[tokio::test]
    async fn consume_body_to_yaml_requres_yaml_content_type() {
        #[derive(Debug, Deserialize)]
        struct Input {
            foo: String,
        }

        let app = Router::new().route("/", post(|input: Yaml<Input>| async { input.0.foo }));

        let client = TestClient::new(app);
        let res = client.post("/").body("foo: bar").await;

        let status = res.status();

        // TODO remove `as_u16()` (?)
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE.as_u16());
    }

    #[tokio::test]
    async fn yaml_content_types() {
        async fn valid_yaml_content_type(content_type: &str) -> bool {
            println!("testing {:?}", content_type);

            let app = Router::new().route("/", post(|Yaml(_): Yaml<Value>| async {}));

            let res = TestClient::new(app)
                .post("/")
                .header("content-type", content_type)
                .body("foo: ")
                .await;

            // TODO res.status() == StatusCode::OK (?)
            res.status() == StatusCode::OK.as_u16()
        }

        assert!(valid_yaml_content_type("application/yaml").await);
        assert!(valid_yaml_content_type("application/yaml;charset=utf-8").await);
        assert!(valid_yaml_content_type("application/yaml; charset=utf-8").await);
        assert!(!valid_yaml_content_type("text/yaml").await);
    }

    #[tokio::test]
    async fn invalid_yaml_syntax() {
        let app = Router::new().route("/", post(|_: Yaml<Value>| async {}));

        let client = TestClient::new(app);
        let res = client
            .post("/")
            .body("- a\nb:")
            .header("content-type", "application/yaml")
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[derive(Deserialize)]
    struct Foo {
        #[allow(dead_code)]
        a: i32,
        #[allow(dead_code)]
        b: Vec<Bar>,
    }

    #[derive(Deserialize)]
    struct Bar {
        #[allow(dead_code)]
        x: i32,
        #[allow(dead_code)]
        y: i32,
    }

    #[tokio::test]
    async fn invalid_yaml_data() {
        let app = Router::new().route("/", post(|_: Yaml<Foo>| async {}));

        let client = TestClient::new(app);
        let res = client
            .post("/")
            .body("a: 1\nb:\n    - x: 2")
            .header("content-type", "application/yaml")
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body_text = res.text().await;
        assert_eq!(
            body_text,
            "Failed to deserialize the YAML body into the target type: b[0]: b[0]: missing field `y` at line 3 column 7"
        );
    }
}

use std::net::{SocketAddr, TcpListener};
use reqwest::RequestBuilder;
use tower_service::Service;
use tower::make::Shared;
use http::{Request, StatusCode};
use axum::body::{Body, HttpBody};
use axum::{BoxError, Router, Server};
use axum::routing::post;
use serde::Deserialize;

use crate::Yaml;

pub struct TestClient {
    client: reqwest::Client,
    addr: SocketAddr,
}

impl TestClient {
    #[allow(clippy::type_repetition_in_bounds)]
    pub(crate) fn new<S, ResBody>(svc: S) -> Self
    where
        S: Service<Request<Body>, Response = http::Response<ResBody>> + Clone + Send + 'static,
        ResBody: HttpBody + Send + 'static,
        ResBody::Data: Send,
        ResBody::Error: Into<BoxError>,
        S::Future: Send,
        S::Error: Into<BoxError>,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();
        println!("Listening on {}", addr);

        tokio::spawn(async move {
            let server = Server::from_tcp(listener).unwrap().serve(Shared::new(svc));
            server.await.expect("server error");
        });

        Self {
            client: reqwest::Client::new(),
            addr,
        }
    }

    pub(crate) fn post(&self, url: &str) -> RequestBuilder {
        self.client.post(format!("http://{}{}", self.addr, url))
    }
}

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
        .body(r#"foo: bar"#)
        .header("content-type", "application/yaml")
        .send()
        .await
        .unwrap();
    let body = res.text().await.unwrap();

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
    let res = client
        .post("/")
        .body(r#"foo: bar"#)
        .send()
        .await
        .unwrap();

    let status = res.status();
    assert!(res.text().await.is_ok());

    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn xml_content_types() {
    async fn valid_yaml_content_type(content_type: &str) -> bool {
        #[derive(Deserialize)]
        struct Value {}

        println!("testing {:?}", content_type);

        let app = Router::new().route("/", post(|Yaml(_): Yaml<Value>| async {}));

        let res = TestClient::new(app)
            .post("/")
            .header("content-type", content_type)
            .body("foo: ")
            .send()
            .await
            .unwrap();

        res.status() == StatusCode::OK
    }

    assert!(valid_yaml_content_type("application/yaml").await);

}
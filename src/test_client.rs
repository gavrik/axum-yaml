// This code was copied from axum crate with minor refactoring, removing unnecessary functionality
// https://github.com/tokio-rs/axum/blob/71eedc6d6cd5fc706ae8d0bcabb49b74e46050f8/axum/src/test_helpers/test_client.rs#L1

use std::{convert::Infallible, future::IntoFuture, net::SocketAddr};

use axum::serve;
use axum_core::{extract::Request, response::Response};
use futures_util::future::BoxFuture;
use http::{HeaderName, HeaderValue, StatusCode};
use tokio::net::TcpListener;
use tower::make::Shared;
use tower_service::Service;

pub(crate) fn spawn_service<S>(svc: S) -> SocketAddr
where
    S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
{
    let std_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    std_listener.set_nonblocking(true).unwrap();
    let listener = TcpListener::from_std(std_listener).unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");

    tokio::spawn(async move {
        serve(listener, Shared::new(svc))
            .await
            .expect("server error")
    });

    addr
}

pub(crate) struct TestClient {
    client: reqwest::Client,
    addr: SocketAddr,
}

impl TestClient {
    pub(crate) fn new<S>(svc: S) -> Self
    where
        S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
        S::Future: Send,
    {
        let addr = spawn_service(svc);

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        TestClient { client, addr }
    }

    pub(crate) fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            builder: self.client.post(format!("http://{}{}", self.addr, url)),
        }
    }
}

pub(crate) struct RequestBuilder {
    builder: reqwest::RequestBuilder,
}

impl RequestBuilder {
    pub(crate) fn body(mut self, body: impl Into<reqwest::Body>) -> Self {
        self.builder = self.builder.body(body);
        self
    }

    pub(crate) fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        // reqwest still uses http 0.2
        let key: HeaderName = key.try_into().map_err(Into::into).unwrap();
        let key = reqwest::header::HeaderName::from_bytes(key.as_ref()).unwrap();

        let value: HeaderValue = value.try_into().map_err(Into::into).unwrap();
        let value = reqwest::header::HeaderValue::from_bytes(value.as_bytes()).unwrap();

        self.builder = self.builder.header(key, value);

        self
    }
}

impl IntoFuture for RequestBuilder {
    type Output = TestResponse;
    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async {
            TestResponse {
                response: self.builder.send().await.unwrap(),
            }
        })
    }
}

#[derive(Debug)]
pub(crate) struct TestResponse {
    response: reqwest::Response,
}

impl TestResponse {
    pub(crate) async fn text(self) -> String {
        self.response.text().await.unwrap()
    }

    pub(crate) fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.response.status().as_u16()).unwrap()
    }
}

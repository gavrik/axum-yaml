# axum-yaml

`axum-yaml` adds YAML features to axum

[![Documentation](https://docs.rs/axum-yaml/badge.svg)](https://docs.rs/axum-yaml)
[![Crates.io](https://img.shields.io/crates/v/axum-yaml)](https://crates.io/crates/axum-yaml)

## Features

* Serialize, Deserialize YAML from request/response

## Usage Example

### Extractor example

When used as an extractor, it can deserialize request bodies into some type that implements [`serde::Deserialize`]. If the request body cannot be parsed, or it does not contain the `Content-Type: application/yaml` header, it will reject the request and return a `400 Bad Request` response.

```rust
use axum::{
    extract,
    routing::post,
    Router,
};
use axum_yaml::Yaml;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    email: String,
    password: String,
}

async fn create_user(Yaml(payload): Yaml<CreateUser>) {
    // payload is a `CreateUser`
}

let app = Router::new().route("/users", post(create_user));
```

> [!NOTE]
> Also, you can construct a `Yaml<T>` from a byte slice (`Yaml::from_bytes()`). Most users should prefer to use the `FromRequest` impl
> but special cases may require first extracting a `Request` into `Bytes` then optionally constructing a `Yaml<T>`.

### Response example

When used as a response, it can serialize any type that implements [`serde::Serialize`] to `YAML`, and will automatically set `Content-Type: application/yaml` header.

```rust
use axum::{
    extract::Path,
    routing::get,
    Router,
};

use axum_yaml::Yaml;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct User {
    id: Uuid,
    username: String,
}

async fn get_user(Path(user_id) : Path<Uuid>) -> Yaml<User> {
    let user = find_user(user_id).await;
    Yaml(user)
}

async fn find_user(user_id: Uuid) -> User {
    // ...
}

let app = Router::new().route("/users/:id", get(get_user));
```

## License

This project is licensed under the MIT license

[docs]: https://docs.rs/axum-yaml
[serde]: https://serde.rs/
[yaml]: https://yaml.org/

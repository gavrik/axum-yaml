//! We use copied macros from axum-core because they are part of a private API that could change in the future
//! https://github.com/tokio-rs/axum/blob/046e7299e1d48503ff78b4ae209b28523f16bd1f/axum-core/src/macros.rs#L1

macro_rules! __log_rejection {
    (
        rejection_type = $ty:ident,
        body_text = $body_text:expr,
        status = $status:expr,
    ) => {
        #[cfg(feature = "tracing")]
        {
            tracing::event!(
                target: "axum_yaml::rejection", // Renamed to "axum_yaml"
                tracing::Level::TRACE,
                status = $status.as_u16(),
                body = $body_text,
                rejection_type = std::any::type_name::<$ty>(),
                "rejecting request",
            );
        }
    };
}
pub(crate) use __log_rejection;

macro_rules! __define_rejection {
    (
        #[status = $status:ident]
        #[body = $body:expr]
        $(#[$m:meta])*
        pub struct $name:ident;
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        #[non_exhaustive]
        pub struct $name;

        impl axum::response::IntoResponse for $name {
            fn into_response(self) -> axum::response::Response {
                super::macros::__log_rejection!(
                    rejection_type = $name,
                    body_text = $body,
                    status = http::StatusCode::$status,
                );
                (self.status(), $body).into_response()
            }
        }

        impl $name {
            /// Get the response body text used for this rejection.
            pub fn body_text(&self) -> String {
                $body.into()
            }

            /// Get the status code used for this rejection.
            pub fn status(&self) -> axum::http::StatusCode {
                axum::http::StatusCode::$status
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", $body)
            }
        }

        impl std::error::Error for $name {}

        impl Default for $name {
            fn default() -> Self {
                Self
            }
        }
    };

    (
        #[status = $status:ident]
        #[body = $body:expr]
        $(#[$m:meta])*
        pub struct $name:ident (Error);
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        pub struct $name(pub(crate) axum::Error);

        impl $name {
            pub(crate) fn from_err<E>(err: E) -> Self
            where
                E: Into<axum::BoxError>,
            {
                Self(axum::Error::new(err))
            }
        }

        impl axum::response::IntoResponse for $name {
            fn into_response(self) -> axum::response::Response {
                super::macros::__log_rejection!(
                    rejection_type = $name,
                    body_text = self.body_text(),
                    status = http::StatusCode::$status,
                );
                (self.status(), self.body_text()).into_response()
            }
        }

        impl $name {
            /// Get the response body text used for this rejection.
            pub fn body_text(&self) -> String {
                format!(concat!($body, ": {}"), self.0).into()
            }

            /// Get the status code used for this rejection.
            pub fn status(&self) -> axum::http::StatusCode {
                axum::http::StatusCode::$status
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", $body)
            }
        }

        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.0)
            }
        }
    };
}
pub(crate) use __define_rejection;

macro_rules! __composite_rejection {
    (
        $(#[$m:meta])*
        pub enum $name:ident {
            $($variant:ident),+
            $(,)?
        }
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        #[non_exhaustive]
        pub enum $name {
            $(
                #[allow(missing_docs)]
                $variant($variant)
            ),+
        }

        impl axum::response::IntoResponse for $name {
            fn into_response(self) -> axum::response::Response {
                match self {
                    $(
                        Self::$variant(inner) => inner.into_response(),
                    )+
                }
            }
        }

        impl $name {
            /// Get the response body text used for this rejection.
            pub fn body_text(&self) -> String {
                match self {
                    $(
                        Self::$variant(inner) => inner.body_text(),
                    )+
                }
            }

            /// Get the status code used for this rejection.
            pub fn status(&self) -> axum::http::StatusCode {
                match self {
                    $(
                        Self::$variant(inner) => inner.status(),
                    )+
                }
            }
        }

        $(
            impl From<$variant> for $name {
                fn from(inner: $variant) -> Self {
                    Self::$variant(inner)
                }
            }
        )+

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant(inner) => write!(f, "{inner}"),
                    )+
                }
            }
        }

        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    $(
                        Self::$variant(inner) => inner.source(),
                    )+
                }
            }
        }
    };
}
pub(crate) use __composite_rejection;

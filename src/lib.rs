#[cfg(feature = "server")]
pub mod config;

#[cfg(feature = "server")]
mod constants {
    pub const SESSION_KEY_LANGUAGE: &str = "language";
}

#[cfg(feature = "server")]
pub mod axum {
    pub use axum::body::Body;
    pub use axum::extract::{Form, Path, Query};
    pub use axum::http::StatusCode;
    pub use axum::response::{IntoResponse, Redirect};
    pub use axum::routing::{get, post};
    pub use axum::{Json, Router};
    pub use tower_sessions::Session;
}

pub mod leptos;

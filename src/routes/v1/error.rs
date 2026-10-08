use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::ntfy;

#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),
    Tera(tera::Error),
    Ntfy(ntfy::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Json(err) => write!(f, "Error while parsing request body: {err}"),
            Error::Tera(err) => write!(f, "Error while rendering templates: {err}"),
            Error::Ntfy(err) => write!(f, "Error dispatching alert to ntfy: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

impl From<tera::Error> for Error {
    fn from(err: tera::Error) -> Error {
        Error::Tera(err)
    }
}

impl From<ntfy::Error> for Error {
    fn from(err: ntfy::Error) -> Error {
        Error::Ntfy(err)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match &self {
            Error::Json(_) => StatusCode::BAD_REQUEST,
            Error::Tera(_) | Error::Ntfy(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        log::error!("request failed: {self}");
        (status, self.to_string()).into_response()
    }
}

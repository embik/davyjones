use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::ntfy;

#[derive(Debug)]
pub enum Error {
    Tera(tera::Error),
    Ntfy(ntfy::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Tera(err) => write!(f, "Error while rendering templates: {err}"),
            Error::Ntfy(err) => write!(f, "Error dispatching alert to ntfy: {err}"),
        }
    }
}

impl std::error::Error for Error {}

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
        log::error!("request failed: {self}");
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

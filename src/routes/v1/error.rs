use actix_web::error;

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

// Use default implementation for `error_response()` method
impl error::ResponseError for Error {}

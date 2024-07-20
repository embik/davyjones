use actix_web::error;

#[derive(Debug)]
pub enum Error {
    Tera(tera::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Tera(_) => write!(f, "Internal Server Error"),
        }
    }
}

impl From<tera::Error> for Error {
    fn from(err: tera::Error) -> Error {
        Error::Tera(err)
    }
}

// Use default implementation for `error_response()` method
impl error::ResponseError for Error {}

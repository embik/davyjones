use awc::{
    error::{PayloadError, SendRequestError},
    http::StatusCode,
};

#[derive(Debug)]
pub enum Error {
    Send(SendRequestError),
    Payload(PayloadError),
    ServerResponse(StatusCode),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Send(err) => write!(f, "error while sending encountered: {err}"),
            Error::Payload(err) => write!(f, "error while reading payload: {err}"),
            Error::ServerResponse(code) => {
                write!(f, "ntfy server returned error code {code}")
            }
        }
    }
}

impl From<SendRequestError> for Error {
    fn from(err: SendRequestError) -> Error {
        Error::Send(err)
    }
}

impl From<PayloadError> for Error {
    fn from(err: PayloadError) -> Error {
        Error::Payload(err)
    }
}

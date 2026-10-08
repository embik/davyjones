use reqwest::StatusCode;

#[derive(Debug)]
pub enum Error {
    Send(reqwest::Error),
    ServerResponse(StatusCode),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Send(err) => write!(f, "error while sending encountered: {err}"),
            Error::ServerResponse(code) => {
                write!(f, "ntfy server returned error code {code}")
            }
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Send(err)
    }
}

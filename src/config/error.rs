// This is an error wrapping all possible errors in this crate.
#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    TOML(toml::de::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IO(err) => write!(f, "IO error encountered: {err}"),
            Error::TOML(err) => write!(f, "error reading TOML: {err}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::TOML(err)
    }
}

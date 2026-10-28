use std::fmt;

#[derive(Debug)]
pub enum Error {
    Cfg(String),
    Io(String),
    Http(String),
    Api { status: u16, message: String },
    Denied,
    Banned,
    Auth(String),
    State(String),
    Sim(String),
    Wallet(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Cfg(msg) => write!(f, "{}", msg),
            Error::Io(msg) => write!(f, "{}", msg),
            Error::Http(msg) => write!(f, "{}", msg),
            Error::Api { status, message } => write!(f, "{} {}", status, message),
            Error::Denied => write!(f, "the session is no longer accepted"),
            Error::Banned => write!(f, "the account is banned"),
            Error::Auth(msg) => write!(f, "{}", msg),
            Error::State(msg) => write!(f, "{}", msg),
            Error::Sim(msg) => write!(f, "{}", msg),
            Error::Wallet(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::State(value.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Http(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

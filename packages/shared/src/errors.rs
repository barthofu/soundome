use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {

    // Generic
    #[error("{0} not found")]
    NotFound(String),
    #[error("no match {0} found for {1}")]
    NoMatch(String, String),
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("config error: {0}")]
    Config(String),

    // HTTP
    #[error("{0} http error: {1}")]
    Http(String, String),

    // Other
    #[error("custom error: {0}")]
    Custom(String),
    #[error("unknown error")]
    Unknown,

    // CLI Parsing
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Json(serde_json::Error),
    #[error("parse error")]
    Parse,
    #[error("missing argument")]
    MissingArg,
    #[error("invalid argument")]
    InvalidArg,
    #[error("process timeout")]
    ProcessTimeout,
    #[error("process error")]
    ExitCode {
        code: i32,
        stderr: String,
    },
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

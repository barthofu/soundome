use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    
    // ============================================================================================
    // Generic errors
    // ============================================================================================
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
    #[error("cache error: {0}")]
    Cache(String),
    #[error("rate limit exceeded: {0}")]
    RateLimit(String),
    #[error("not implemented error: {0}")]
    NotImplemented(String),

    // ============================================================================================
    // Domain errors
    // ============================================================================================

    // Track
    #[error("track not found: {0}")]
    TrackNotFound(String),
    #[error("track already exists: {0}")]
    TrackExists(String),
    #[error("track download failed: {0}")]
    TrackDownloadFailed(String),
    #[error("track processing failed: {0}")]
    TrackProcessingFailed(String),
    #[error("track metadata error: {0}")]
    TrackMetadataError(String),
    
    #[error("{0} provider is not available")]
    ProviderUnavailable(String),

    // ============================================================================================
    // Technical errors
    // ============================================================================================

    // HTTP
    #[error("{0} http error: {1}")]
    Http(String, String),

    // Database
    #[error("database error: {0}")]
    Database(String),

    // Other
    #[error("custom error: {0}")]
    Custom(String),
    #[error("unknown error")]
    Unknown,

    // AI
    #[error("no AI backend configured")]
    NoAIBackend,

    // String
    #[error("string template error: {0}")]
    TemplateRenderingError(tinytemplate::error::Error),
    #[error("invalid path: {0}")]
    InvalidPath(std::path::PathBuf),

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
    ExitCode { code: i32, stderr: String },
}

#[cfg(feature = "diesel_integration")]
impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::Database(format!("Database error: {}", err))
    }
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

impl From<tinytemplate::error::Error> for Error {
    fn from(err: tinytemplate::error::Error) -> Self {
        Error::TemplateRenderingError(err)
    }
}

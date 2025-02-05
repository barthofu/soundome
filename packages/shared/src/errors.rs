#[derive(Debug)]
pub enum Error {
    
    // API errors
    NotFound,
    BadURL,
    InternalServer, 
    Network, // the network is the cause of the error

    // CLI parsing
    Io(std::io::Error),
    Json(serde_json::Error),
    Parse,
    MissingArg,
    InvalidArg,
    ProcessTimeout,
    ExitCode {
        code: i32,
        stderr: String,
    },

    // Other
    Config,
    Other,
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
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid invocation {0}")]
    Invoke(String),
    #[error("Invalid compute {0}")]
    Function(String),

    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

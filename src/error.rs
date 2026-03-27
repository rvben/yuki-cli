use thiserror::Error;

#[derive(Debug, Error)]
pub enum YukiError {
    #[error("authentication failed: {0}")]
    AuthFailed(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("rate limited: 1000 calls/day exceeded")]
    RateLimited,

    #[error("HTTP error {status}: {body}")]
    Http { status: u16, body: String },

    #[error("SOAP fault [{code}]: {message}")]
    SoapFault { code: String, message: String },

    #[error("configuration error: {0}")]
    Config(String),

    #[error("XML error: {0}")]
    Xml(String),

    #[error("{0}")]
    Request(#[from] reqwest::Error),
}

impl YukiError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::AuthFailed(_) => 2,
            Self::NotFound(_) => 3,
            Self::RateLimited => 4,
            _ => 1,
        }
    }
}

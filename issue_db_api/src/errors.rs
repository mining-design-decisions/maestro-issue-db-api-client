#[allow(unused)]
#[derive(Debug)]
pub enum APIError {
    InvalidCredentials,
    InvalidToken,
    NotAuthorized,
    HTTPError{message: String, status_code: u16},
    IDParsingError(String),
    LibraryError(String),
    GenericError(String)
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            APIError::InvalidCredentials => write!(f, "Invalid credentials"),
            APIError::InvalidToken => write!(f, "Invalid token"),
            APIError::NotAuthorized => write!(f, "Calling authorized endpoint without token"),
            APIError::HTTPError { message, status_code } => {
                write!(f, "Error in HTTP (code = {status_code}): {message}")
            },
            APIError::IDParsingError(msg) => write!(f, "Error while parsing ObjectID: {msg}"),
            APIError::LibraryError(msg) => write!(f, "Internal error in library: {msg}"),
            APIError::GenericError(msg) => write!(f, "Error: {msg}")
        }
    }
}

impl std::error::Error for APIError {}

impl From<reqwest::Error> for APIError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_status() {
            let code = value.status().expect("Expected status code");
            if code == reqwest::StatusCode::UNAUTHORIZED {
                APIError::InvalidCredentials
            } else {
                APIError::HTTPError{message: value.to_string(), status_code: code.as_u16()}
            }
        } else {
            APIError::LibraryError(value.to_string())
        }
    }
}

impl From<std::io::Error> for APIError {
    fn from(value: std::io::Error) -> Self {
        APIError::GenericError(value.to_string())
    }
}

impl From<serde_json::Error> for APIError {
    fn from(value: serde_json::Error) -> Self {
        APIError::GenericError(value.to_string())
    }
}

impl From<crate::util::CacheLockError> for APIError {
    fn from(value: crate::util::CacheLockError) -> Self {
        APIError::LibraryError(value.to_string())
    }
}

pub type APIResult<T> = Result<T, APIError>;

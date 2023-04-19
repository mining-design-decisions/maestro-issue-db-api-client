//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Custom errors
//////////////////////////////////////////////////////////////////////////////////////////////////

use std::error::Error;

#[allow(unused)]
#[derive(Debug)]
pub struct AuthenticationError {}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Attempted to invoke an authenticated endpoint without token")
    }
}

impl Error for AuthenticationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}


#[allow(unused)]
#[derive(Debug)]
pub struct IDParsingError {
    pub msg: String
}

impl std::fmt::Display for IDParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse ObjectID: {}", self.msg)
    }
}

impl Error for IDParsingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}


#[allow(unused)]
#[derive(Debug)]
pub struct APIError {
    msg: String
}

impl APIError {
    pub fn new(msg: String) -> Self {
        APIError{msg}
    }
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in API: {}", self.msg)
    }
}

impl Error for APIError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
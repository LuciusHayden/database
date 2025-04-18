use std::fmt;
use std::io;


#[derive(Debug)]
pub enum DatabaseError {
    SyntaxError(String),
    PermissionDenied(String),
    CollectionNotFound(String),
    UserError(String),
    SerializationError(String),
    IOError(io::Error),
    CollectionError(String),
    Other(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::SyntaxError(msg) => write!(f, "Syntax Error: {}", msg),
            DatabaseError::CollectionNotFound(name) => write!(f, "Collection not found: {}", name),
            DatabaseError::PermissionDenied(permission) => write!(f, "Permission not high enough: {}", permission),
            DatabaseError::UserError(error) => write!(f, "Login Error: {}", error), 
            DatabaseError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            DatabaseError::IOError(err) => write!(f, "IO error: {}", err),
            DatabaseError::CollectionError(msg) => write!(f, "Collection Error: {}", msg),
            DatabaseError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> Self {
        DatabaseError::IOError(err)
    }
}

impl From<bincode::Error> for DatabaseError {
    fn from(err: bincode::Error) -> Self {
        DatabaseError::SerializationError(format!("bincode: {}", err))
    }
}


impl From<bcrypt::BcryptError> for DatabaseError {
    fn from(err: bcrypt::BcryptError) -> Self {
        DatabaseError::Other(format!("bcrypt: {}", err))
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from( err: serde_json::Error) -> Self {
        DatabaseError::Other(format!("serde_json: {}", err))
    }
}

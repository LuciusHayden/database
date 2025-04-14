use std::fmt;
use std::io;


#[derive(Debug)]
pub enum DatabaseError {
    PermissionDenied(String),
    CollectionNotFound(String),
    UserError(String),
    SerializationError(String),
    IOError(io::Error),
    Other(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::CollectionNotFound(name) => write!(f, "Collection not found: {}", name),
            DatabaseError::PermissionDenied(permission) => write!(f, "Permission not high enough: {}", permission),
            DatabaseError::UserError(error) => write!(f, "Login Error: {}", error), 
            DatabaseError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            DatabaseError::IOError(err) => write!(f, "IO error: {}", err),
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
        DatabaseError::SerializationError(err.to_string())
    }
}


impl From<bcrypt::BcryptError> for DatabaseError {
    fn from(err: bcrypt::BcryptError) -> Self {
        DatabaseError::Other(err.to_string())
    }
}

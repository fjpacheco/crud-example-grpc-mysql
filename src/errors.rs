#[derive(Debug, PartialEq)]
pub enum TypeErrorKinsper {
    ConnectionError,
    MySqlError,
    UpdateSchemeError,
    InvalidEmail,
    InvalidId,
    InternalValidationError,
    NotFound,
    AlreadyExists,
}

impl std::fmt::Display for TypeErrorKinsper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeErrorKinsper::ConnectionError => write!(f, "Error connection error"),
            TypeErrorKinsper::MySqlError => write!(f, "Error MySql"),
            TypeErrorKinsper::UpdateSchemeError => write!(f, "Error update scheme"),
            TypeErrorKinsper::InvalidEmail => write!(f, "Error invalid email"),
            TypeErrorKinsper::InvalidId => write!(f, "Error invalid id"),
            TypeErrorKinsper::InternalValidationError => write!(f, "Error internal validation"),
            TypeErrorKinsper::NotFound => write!(f, "Error not found"),
            TypeErrorKinsper::AlreadyExists => write!(f, "Error already exists"),
        }
    }
}

#[derive(Debug)]
pub struct ErrorKinsper {
    pub type_error: TypeErrorKinsper,
    pub message: String,
}

impl ErrorKinsper {
    pub fn new(type_error: TypeErrorKinsper, message: String) -> Self {
        ErrorKinsper {
            type_error,
            message,
        }
    }
}

impl std::fmt::Display for ErrorKinsper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.type_error, self.message)
    }
}

impl From<sqlx::Error> for ErrorKinsper {
    fn from(err: sqlx::Error) -> Self {
        let (kind, msg) = match err {
            sqlx::Error::Database(e) if e.to_string().contains("Duplicate entry") => (
                TypeErrorKinsper::AlreadyExists,
                "Error duplicate entry".to_string(),
            ),
            sqlx::Error::RowNotFound => (
                TypeErrorKinsper::NotFound,
                "Error user not found".to_string(),
            ),
            _ => (
                TypeErrorKinsper::MySqlError,
                format!("Error from MySql: {}", err),
            ),
        };

        ErrorKinsper::new(kind, msg)
    }
}

use tonic::Status;

impl From<ErrorKinsper> for Status {
    fn from(err: ErrorKinsper) -> Self {
        match err.type_error {
            TypeErrorKinsper::ConnectionError => Status::internal(err.message),
            TypeErrorKinsper::MySqlError => Status::internal(err.message),
            TypeErrorKinsper::UpdateSchemeError => Status::internal(err.message),
            TypeErrorKinsper::InvalidEmail => Status::invalid_argument(err.message),
            TypeErrorKinsper::InvalidId => Status::invalid_argument(err.message),
            TypeErrorKinsper::InternalValidationError => Status::internal(err.message),
            TypeErrorKinsper::NotFound => Status::not_found(err.message),
            TypeErrorKinsper::AlreadyExists => Status::already_exists(err.message),
        }
    }
}

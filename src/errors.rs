#[derive(Debug, PartialEq)]
pub enum ErrorKinsper {
    InternalServer(String),
    InvalidUri(String),
    ConnectionError(String),
    MySqlError(String),
    UpdateSchemeError(String),
    InvalidEmail(String),
    InvalidId(String),
    InternalValidationError(String),
    NotFound(String),
    AlreadyExists(String),
    Unknown,
}

impl From<sqlx::Error> for ErrorKinsper {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(e) if e.to_string().contains("Duplicate entry") => {
                ErrorKinsper::AlreadyExists("Error duplicate entry".to_string())
            }
            sqlx::Error::RowNotFound => ErrorKinsper::NotFound("Error user not found".to_string()),
            _ => ErrorKinsper::MySqlError(format!("Error from MySql: {}", err)),
        }
    }
}

use tonic::Status;

impl From<ErrorKinsper> for Status {
    fn from(err: ErrorKinsper) -> Self {
        match err {
            ErrorKinsper::InternalServer(msg) => Status::internal(msg),
            ErrorKinsper::InvalidUri(msg) => Status::internal(msg),
            ErrorKinsper::ConnectionError(msg) => Status::internal(msg),
            ErrorKinsper::MySqlError(msg) => Status::internal(msg),
            ErrorKinsper::UpdateSchemeError(msg) => Status::internal(msg),
            ErrorKinsper::InvalidEmail(msg) => Status::invalid_argument(msg),
            ErrorKinsper::InvalidId(msg) => Status::invalid_argument(msg),
            ErrorKinsper::InternalValidationError(msg) => Status::internal(msg),
            ErrorKinsper::NotFound(msg) => Status::not_found(msg),
            ErrorKinsper::AlreadyExists(msg) => Status::already_exists(msg),
            ErrorKinsper::Unknown => Status::internal("Unknown error"),
        }
    }
}

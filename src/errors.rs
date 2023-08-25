#[derive(Debug)]
pub enum TypeErrorKinsper {
    ConnectionError,
    MySqlError,
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

impl From<sqlx::Error> for ErrorKinsper {
    fn from(err: sqlx::Error) -> Self {
        ErrorKinsper::new(
            TypeErrorKinsper::MySqlError,
            format!("Error from MySql: {}", err),
        )
    }
}

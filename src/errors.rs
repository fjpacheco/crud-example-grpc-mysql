#[derive(Debug)]
pub enum TypeErrorKinsper{
    ConnectionError,
}

#[derive(Debug)]
pub struct ErrorKinsper{
    pub type_error: TypeErrorKinsper,
    pub message: String,
}

impl ErrorKinsper{
    pub fn new(type_error: TypeErrorKinsper, message: String) -> Self {
        ErrorKinsper{
            type_error,
            message,
        }
    }
}
pub mod data;
pub mod errors;
pub mod handler_server;

use std::env;

use errors::ErrorKinsper;
use tonic::Status;

const DEFAULT_LEVEL_LOG: log::LevelFilter = log::LevelFilter::Info;
pub const SERVER_LOCALPORT: u16 = 50051;
pub const SERVER_LOCALHOST: &str = "127.0.0.1";
pub const QUERY_LIMIT_CLIENT: &str = "1024";
pub const LIMIT_STREAM_QUEUE: usize = 1024;

pub const MAX_T_SCHEDULING_USERS_TEST: usize = 10;
pub const MAX_USERS_TEST: usize = 1024;

pub fn initialize_logging() {
    env_logger::builder()
        .filter(
            None,
            env::var("RUST_LOG")
                .unwrap_or_default()
                .parse::<log::LevelFilter>()
                .unwrap_or(DEFAULT_LEVEL_LOG),
        )
        .format_timestamp(None)
        .init();
}

pub fn validate_mail(mail: &str) -> Result<(), Status> {
    regex::Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})",
    )
    .map_err(|_| ErrorKinsper::InternalValidationError("Error in validations.".to_string()))?
    .is_match(mail)
    .then_some(())
    .ok_or_else(|| ErrorKinsper::InvalidEmail("Invalid email.".to_string()).into())
}

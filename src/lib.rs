pub mod data;
pub mod errors;
use std::env;

use errors::TypeErrorKinsper;
use tonic::Status;

const DEFAULT_LEVEL_LOG: log::LevelFilter = log::LevelFilter::Info;
pub const SERVER_LOCALPORT: u16 = 50051;
pub const SERVER_LOCALHOST: &str = "127.0.0.1";
pub const MAX_USERS_TEST: usize = 1024;
pub const QUERY_LIMIT_CLIENT: &str = "10";
pub const LIMIT_STREAM_QUEUE: usize = 10;

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
    .map_err(|_| Status::internal(TypeErrorKinsper::InternalValidationError.to_string()))
    .and_then(|re| {
        re.is_match(mail)
            .then_some(())
            .ok_or_else(|| Status::invalid_argument(TypeErrorKinsper::InvalidEmail.to_string()))
    })
}

pub mod data;
pub mod errors;
use std::env;

const DEFAULT_LEVEL_LOG: log::LevelFilter = log::LevelFilter::Info;
pub const SERVER_LOCALPORT: u16 = 50051;
pub const SERVER_LOCALHOST: &str = "127.0.0.1";

pub const MAX_USERS_TEST: usize = 1024;
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

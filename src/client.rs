use std::env;

use dotenv::dotenv;
use log::LevelFilter;
use user_service::user_service_client::UserServiceClient;

pub mod user_service {
    // Note: The token passed to the include_proto macro (in our case "routeguide") is
    // the name of the package declared in our .proto file, not a filename, e.g "routeguide.rs".
    tonic::include_proto!("user_service");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::builder()
        .filter(
            None,
            env::var("RUST_LOG")
                .unwrap_or_default()
                .parse::<LevelFilter>()
                .unwrap_or(LevelFilter::Info),
        )
        .format_timestamp(None)
        .init();

    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;
    log::info!("Connected to server.");

    let request_get_user = tonic::Request::new(user_service::UserId {
        value: "1".to_string(),
    });

    log::info!("[1] Sending request {:?}", request_get_user);
    let response = client.get_user(request_get_user).await?;
    log::info!("[1] Response: {:?}", response);

    Ok(())
}

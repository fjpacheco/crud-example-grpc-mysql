use std::env;

use dotenv::dotenv;
use log::LevelFilter;
use user_service::user_service_client::UserServiceClient;

pub mod user_service {
    use tonic::include_proto;

    // Note: The token passed to the include_proto macro (in our case "routeguide") is
    // the name of the package declared in our .proto file, not a filename, e.g "routeguide.rs".
    include_proto!("user_service");
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

    let mut request_get_user = tonic::Request::new(user_service::UserRequest {
        id: "23".to_string(),
    });

    log::info!("[GET_USER] [1] Sending request {:?}", request_get_user);
    let response = client.get_user(request_get_user).await;
    log::info!("[GET_USER] [1] Response: {:?}", response);

    let request_create_user = tonic::Request::new(user_service::CreateUserRequest {
        id: "23".to_string(),
        name: "John Doe".to_string(),
        mail: "jhon@test.com".to_string(),
    });

    log::info!(
        "[CREATE_USER] [1] Sending request {:?}",
        request_create_user
    );
    let response = client.create_user(request_create_user).await;
    log::info!("[CREATE_USER] [1] Response: {:?}", response);

    request_get_user = tonic::Request::new(user_service::UserRequest {
        id: "23".to_string(),
    });

    log::info!("[GET_USER] [2] Sending request {:?}", request_get_user);
    let response = client.get_user(request_get_user).await;
    log::info!("[GET_USER] [2] Response: {:?}", response);

    Ok(())
}

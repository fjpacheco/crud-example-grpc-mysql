use dotenv::dotenv;
use futures::stream::StreamExt;
use kinsper_rust_test::{initialize_logging, MAX_USERS_TEST, SERVER_LOCALHOST, SERVER_LOCALPORT};
use rand::Rng;
use user_service::user_service_client::UserServiceClient;

use crate::user_service::UserId;

pub mod user_service {
    // Note: The token passed to the include_proto macro (in our case "routeguide") is
    // the name of the package declared in our .proto file, not a filename, e.g "routeguide.rs".
    tonic::include_proto!("user_service");
}

// #[tokio::main] // by default, it uses 4 threads
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    initialize_logging();
    let addr = format!("http://{}:{}", SERVER_LOCALHOST, SERVER_LOCALPORT);

    let fetches = futures::stream::iter((0..MAX_USERS_TEST).map(|user_id| {
        // let user_rng_id = 10_usize;
        let user_rng_id = rand::thread_rng().gen_range(0..MAX_USERS_TEST);
        let client = UserServiceClient::connect(addr.clone());

        tokio::spawn(async move {
            let request_get_user = tonic::Request::new(user_service::GetUserRequest {
                id: Some(UserId {
                    id: user_rng_id.to_string(),
                }),
            });

            let request_create_user_1 = tonic::Request::new(user_service::CreateUserRequest {
                id: Some(UserId {
                    id: user_rng_id.to_string(),
                }),
                name: format!("John Doe A {}", user_rng_id),
                mail: String::from("Jhon@mail.com"),
            });

            let request_create_user_2 = tonic::Request::new(user_service::CreateUserRequest {
                id: Some(UserId {
                    id: user_rng_id.to_string(),
                }),
                name: format!("John Doe B {}", user_rng_id),
                mail: String::from("Jhon2@mail.com"),
            });

            let request_update_name_user =
                tonic::Request::new(user_service::UpdateUserNameRequest {
                    id: Some(UserId {
                        id: user_rng_id.to_string(),
                    }),
                    name: format!("John Doe Updated by {}", user_id),
                });

            if let Ok(mut client) = client.await {
                let _ = client.get_user(request_get_user).await;
                let _ = client.create_user(request_create_user_1).await;
                let _ = client.create_user(request_create_user_2).await;
                let _ = client.update_name_user(request_update_name_user).await;
            }

            log::info!(
                "END_REQ_ID_{}] | [CURRENT_THREAD: {:?}] | [THREAD_NAME: {:?}]",
                user_id,
                std::thread::current().id(),
                std::thread::current().name().unwrap()
            );
        })
    }));

    fetches.buffer_unordered(5).collect::<Vec<_>>().await;

    Ok(())
}

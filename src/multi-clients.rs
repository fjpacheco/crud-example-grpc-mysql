use dotenv::dotenv;
use futures::stream::StreamExt;
use kinsper_rust_test::{
    errors::ErrorKinsper, initialize_logging, MAX_T_SCHEDULING_USERS_TEST, MAX_USERS_TEST,
    SERVER_LOCALHOST, SERVER_LOCALPORT,
};
use rand::Rng;
use user_service::{user_service_client::UserServiceClient, GetAllUserRequest};

use crate::user_service::UserId;

pub mod user_service {
    // Note: The token passed to the include_proto macro (in our case "routeguide") is
    // the name of the package declared in our .proto file, not a filename, e.g "routeguide.rs".
    tonic::include_proto!("user_service");
}

// #[tokio::main] // by default, it uses 4 threads
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), ErrorKinsper> {
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
                mail: String::from("jhon@mail.com"),
            });

            let request_create_user_2 = tonic::Request::new(user_service::CreateUserRequest {
                id: Some(UserId {
                    id: user_rng_id.to_string(),
                }),
                name: format!("John Doe B {}", user_rng_id),
                mail: String::from("jhon2@mail.com"),
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

    fetches
        .buffer_unordered(MAX_T_SCHEDULING_USERS_TEST)
        .collect::<Vec<_>>()
        .await;

    let mut client = UserServiceClient::connect(addr)
        .await
        .map_err(|_| ErrorKinsper::InternalServer("Error connecting to server".to_string()))?;

    print!("10 users from the server: ");
    let mut stream = client
        .get_all_users(GetAllUserRequest { limit: 10 })
        .await
        .map_err(|e| {
            if e.code() == tonic::Code::NotFound {
                ErrorKinsper::NotFound("User not found".to_string())
            } else {
                ErrorKinsper::InternalServer(e.to_string())
            }
        })?
        .into_inner();

    while let Some(user) = stream
        .message()
        .await
        .map_err(|e| ErrorKinsper::InternalServer(e.to_string()))?
    {
        println!("User = {:?}", user);
    }

    Ok(())
}

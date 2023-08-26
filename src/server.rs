use dotenv::dotenv;
use kinsper_rust_test::data::model::UserModel;
use log::LevelFilter;
use std::env;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

use user_service::user_service_server::{UserService, UserServiceServer};
use user_service::{User, UserId};

pub mod user_service {
    tonic::include_proto!("user_service");
}

#[derive(Default)]
pub struct MyUserService {
    mock_users: Arc<Vec<UserModel>>, // TODO! Arc ... async arc?
}

#[tonic::async_trait]
impl UserService for MyUserService {
    async fn get_user(&self, request: Request<UserId>) -> Result<Response<User>, Status> {
        log::info!("Got a request from {:?}", request.remote_addr());

        let user = self
            .mock_users
            .iter()
            .find(|u| u.id == request.get_ref().value);

        match user {
            Some(user) => {
                let user = User {
                    id: Some(UserId {
                        value: user.id.clone(),
                    }),
                    name: user.name.clone(),
                    mail: user.mail.clone(),
                };
                Ok(Response::new(user))
            }
            None => Err(Status::not_found("User not found")),
        }
    }
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

    let addr = "[::1]:50051".parse().unwrap();

    let users = MyUserService {
        mock_users: Arc::new(vec![
            UserModel {
                id: "1".to_string(),
                name: "Fede".to_string(),
                mail: "fede@test.com".to_string(),
            },
            UserModel {
                id: "2".to_string(),
                name: "Abel".to_string(),
                mail: "abel@test.com".to_string(),
            },
        ]),
    };

    log::info!("Listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(users))
        .serve(addr)
        .await?;

    Ok(())
}

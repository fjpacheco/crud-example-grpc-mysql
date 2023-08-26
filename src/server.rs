use dotenv::dotenv;
use kinsper_rust_test::data::context::Database;
use kinsper_rust_test::data::scheme::CreateUserScheme;
use log::LevelFilter;
use std::env;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

use user_service::user_service_server::{UserService, UserServiceServer};
use user_service::{CreateUserRequest, UserRequest, UserResponse};

pub mod user_service {
    use tonic::include_proto;

    include_proto!("user_service");
}

pub struct MyUserService {
    // Similarly, it isn't a good idea to hold a traditional non-futures-aware lock
    // across an .await, as it can cause the threadpool to lock up: one task could take
    // out a lock, .await and yield to the executor, allowing another task to attempt to
    // take the lock and cause a DEADLOCK. To avoid this, use the
    // Mutex in futures::lock rather than the one from std::sync.
    // REF: https://rust-lang.github.io/async-book/03_async_await/01_chapter.html?highlight=lock#awaiting-on-a-multithreaded-executor

    // The primary use case for the async mutex is to provide shared mutable access to
    // IO resources such as a database connection. If the value behind the mutex is just data,
    // it’s usually appropriate to use a blocking mutex such as the one in the standard library
    // or parking_lot.
    // REF: https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html#:~:text=The%20primary%20use%20case%20for%20the%20async%20mutex%20is%20to%20provide%20shared%20mutable%20access%20to%20IO%20resources%20such%20as%20a%20database%20connection.%20If%20the%20value%20behind%20the%20mutex%20is%20just%20data%2C%20it%E2%80%99s%20usually%20appropriate%20to%20use%20a%20blocking%20mutex%20such%20as%20the%20one%20in%20the%20standard%20library%20or%20parking_lot.
    db_context: Arc<Database>,
}

#[tonic::async_trait]
impl UserService for MyUserService {
    async fn get_user(
        &self,
        request: Request<UserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        log::info!("[GET_USER] Got a request from {:?}", request.remote_addr());

        let id = request.get_ref().id.clone();
        let user = self.db_context.get_user_by_id(&id).await;

        match user {
            Ok(user) => {
                let user = UserResponse {
                    id: user.id.clone(),
                    name: user.name.clone(),
                    mail: user.mail.clone(),
                };
                Ok(Response::new(user))
            }
            Err(_) => Err(Status::not_found("User not found")),
        }
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        log::info!(
            "[CREATE_USER] Got a request from {:?}",
            request.remote_addr()
        );

        let user = CreateUserScheme {
            id: request.get_ref().id.clone(),
            name: request.get_ref().name.clone(),
            mail: request.get_ref().mail.clone(),
        };

        match self.db_context.add_user(&user).await {
            Ok(_) => {
                let user = UserResponse {
                    id: user.id.clone(),
                    name: user.name.clone(),
                    mail: user.mail.clone(),
                };
                Ok(Response::new(user))
            }
            Err(_) => Err(Status::internal("Couldn't create user")),
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

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_context = Database::connect(&database_url).await.unwrap();

    let users = MyUserService {
        db_context: Arc::new(db_context),
    };

    log::info!("Listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(users))
        .serve(addr)
        .await?;

    Ok(())
}

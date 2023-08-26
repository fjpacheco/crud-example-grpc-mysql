use dotenv::dotenv;
use kinsper_rust_test::data::context::Database;
use kinsper_rust_test::data::scheme::{CreateUserScheme, UpdateUserSchema};
use kinsper_rust_test::errors::ErrorKinsper;
use log::LevelFilter;
use std::env;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

use user_service::user_service_server::{UserService, UserServiceServer};
use user_service::{
    CreateUserRequest, CreateUserResponse, DeleteUserRequest, DeleteUserResponse, GetUserRequest,
    GetUserResponse, UpdateUserMailRequest, UpdateUserMailResponse, UpdateUserNameRequest,
    UpdateUserNameResponse, UserId,
};

pub mod user_service {
    tonic::include_proto!("user_service");
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
    // mock_users : Arc<tokio::sync::Mutex<HashMap<String, User>>>,
    db_context: Arc<Database>,
}

impl MyUserService {
    async fn update_user_helper<F, R>(
        &self,
        id: &str,
        schema_creator: F,
        response_creator: fn() -> R,
    ) -> Result<Response<R>, Status>
    where
        F: FnOnce() -> Result<UpdateUserSchema, ErrorKinsper>,
    {
        let updated_schema =
            schema_creator().map_err(|_| Status::internal("Couldn't create update schema"))?;
        self.db_context
            .update_user(id, &updated_schema)
            .await
            .map(|_| Response::new(response_creator()))
            .map_err(|_| Status::internal("Couldn't update user"))
    }

    fn id_to_str<'a>(&self, id: &'a Option<UserId>) -> Result<&'a str, Status> {
        match id {
            Some(id) => Ok(&id.id),
            None => Err(Status::internal("Invalid request: Missing ID")),
        }
    }
}

#[tonic::async_trait]
impl UserService for MyUserService {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let id = self.id_to_str(&request.get_ref().id)?;
        log::info!("[GET_USER] Got a request from {:?}", request.remote_addr());

        let user = self.db_context.get_user_by_id(id).await;

        match user {
            Ok(user) => Ok(Response::new(GetUserResponse {
                id: Some(UserId { id: user.id }),
                name: user.name,
                mail: user.mail,
            })),
            Err(_) => Err(Status::not_found("User not found")),
        }
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        log::info!(
            "[CREATE_USER] Got a request from {:?}",
            request.remote_addr()
        );

        let req = request.get_ref();
        let user = CreateUserScheme {
            id: self.id_to_str(&req.id)?.to_string(),
            name: req.name.clone(),
            mail: req.mail.clone(),
        };

        self.db_context
            .add_user(&user)
            .await
            .map(|_| Response::new(CreateUserResponse {}))
            .map_err(|_| Status::internal("Couldn't create user"))
    }

    async fn update_name_user(
        &self,
        request: Request<UpdateUserNameRequest>,
    ) -> Result<Response<UpdateUserNameResponse>, Status> {
        let req: &UpdateUserNameRequest = request.get_ref();
        let id = self.id_to_str(&req.id)?;

        log::info!(
            "[UPDATE_USER_MAIL] Got a request from {:?}",
            request.remote_addr()
        );

        let name = req.name.clone();
        self.update_user_helper(
            &id,
            || UpdateUserSchema::new().with_name(name).finalize(),
            || UpdateUserNameResponse {},
        )
        .await
    }

    async fn update_mail_user(
        &self,
        request: Request<UpdateUserMailRequest>,
    ) -> Result<Response<UpdateUserMailResponse>, Status> {
        let req: &UpdateUserMailRequest = request.get_ref();
        let id = self.id_to_str(&req.id)?;

        log::info!(
            "[UPDATE_USER_MAIL] Got a request from {:?}",
            request.remote_addr()
        );

        let mail = req.mail.clone();
        self.update_user_helper(
            id,
            || UpdateUserSchema::new().with_mail(mail).finalize(),
            || UpdateUserMailResponse {},
        )
        .await
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let id = self.id_to_str(&request.get_ref().id)?;

        log::info!(
            "[DELETE_USER] Got a request from {:?}",
            request.remote_addr()
        );

        self.db_context
            .delete_user(id)
            .await
            .map(|_| Response::new(DeleteUserResponse {}))
            .map_err(|_| Status::internal("Couldn't delete user"))
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

use crate::data::context::Database;
use crate::data::scheme::{CreateUserScheme, UpdateUserSchema};
use crate::errors::ErrorKinsper;
use crate::{validate_mail, LIMIT_STREAM_QUEUE};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use user_service::user_service_server::UserService;
use user_service::{
    CreateUserRequest, CreateUserResponse, DeleteUserRequest, DeleteUserResponse,
    GetAllUserRequest, GetUserRequest, GetUserResponse, ResetUserTableRequest,
    ResetUserTableResponse, UpdateUserMailRequest, UpdateUserMailResponse, UpdateUserNameRequest,
    UpdateUserNameResponse, UserId,
};

pub mod user_service {
    tonic::include_proto!("user_service");
}

pub struct MyUserService {
    pub db_context: Database,
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
        let updated_schema = schema_creator()?;

        // tokio::time::sleep(std::time::Duration::from_secs(5)).await; // For play with concurrency with quantitiy workers on tokio runtime and buffer_unordered
        // std::thread::sleep(std::time::Duration::from_secs(5)); // For visualize ops blocking in runtime thread

        Ok(self
            .db_context
            .update_user(id, &updated_schema)
            .await
            .map(|_| Response::new(response_creator()))?)
    }

    fn id_to_str<'a>(&self, id: &'a Option<UserId>) -> Result<&'a str, Status> {
        match id {
            Some(id) => Ok(&id.id),
            None => Err(ErrorKinsper::InvalidId("Invalid id".to_string()).into()),
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

        let user = self.db_context.get_user_by_id(id).await?;

        Ok(Response::new(GetUserResponse {
            id: Some(UserId { id: user.id }),
            name: user.name,
            mail: user.mail,
        }))
    }

    type GetAllUsersStream = ReceiverStream<Result<GetUserResponse, Status>>;

    async fn get_all_users(
        &self,
        request: Request<GetAllUserRequest>,
    ) -> Result<Response<Self::GetAllUsersStream>, Status> {
        log::info!("[GET_USERS] Got a request from {:?}", request.remote_addr());

        let (tx, rx) = mpsc::channel(LIMIT_STREAM_QUEUE);

        let users = self
            .db_context
            .get_users(Some(request.get_ref().limit))
            .await?;
        tokio::spawn(async move {
            for user in users {
                if tx
                    .send(Ok(GetUserResponse {
                        id: Some(UserId { id: user.id }),
                        name: user.name,
                        mail: user.mail,
                    }))
                    .await
                    .is_err()
                {
                    log::error!("Channel send error");
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.get_ref();
        validate_mail(&req.mail)?;

        log::info!(
            "[CREATE_USER] Got a request from {:?}",
            request.remote_addr()
        );

        let user = CreateUserScheme {
            id: self.id_to_str(&req.id)?.to_string(),
            name: req.name.clone(),
            mail: req.mail.clone(),
        };

        Ok(self
            .db_context
            .add_user(&user)
            .await
            .map(|_| Response::new(CreateUserResponse {}))?)
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
            id,
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
        validate_mail(&req.mail)?;

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

        Ok(self
            .db_context
            .delete_user(id)
            .await
            .map(|_| Response::new(DeleteUserResponse {}))?)
    }

    async fn reset_user_table(
        &self,
        request: Request<ResetUserTableRequest>,
    ) -> Result<Response<ResetUserTableResponse>, Status> {
        log::info!(
            "[RESET_USER_TABLE] Got a request from {:?}",
            request.remote_addr()
        );

        Ok(self
            .db_context
            .reset_table()
            .await
            .map(|_| Response::new(ResetUserTableResponse {}))?)
    }
}

#[cfg(test)]
mod test_tonic_server {
    use std::sync::Arc;

    use crate::handler_server::user_service::user_service_server::UserServiceServer;
    use futures_util::Future;
    use tempfile::NamedTempFile;
    use tokio::net::{UnixListener, UnixStream};
    use tokio_stream::wrappers::UnixListenerStream;
    use tonic::transport::{Channel, Endpoint, Server, Uri};
    use tonic::Request;
    use tower::service_fn;

    use crate::data::context::Database;
    use crate::data::handler::handler_tests::TEST_COUNTER;
    use crate::errors::ErrorKinsper;
    use crate::handler_server::MyUserService;
    use user_service::{
        user_service_client::UserServiceClient, CreateUserRequest, GetAllUserRequest,
        GetUserRequest, ResetUserTableRequest, UpdateUserMailRequest, UpdateUserNameRequest,
        UserId,
    };
    pub mod user_service {
        tonic::include_proto!("user_service");
    }
    use std::sync::atomic::Ordering;

    // https://stackoverflow.com/questions/69845664/how-to-integration-test-tonic-application
    async fn server_and_client_stub() -> (impl Future<Output = ()>, UserServiceClient<Channel>) {
        let socket = NamedTempFile::new().unwrap();
        let socket = Arc::new(socket.into_temp_path());
        std::fs::remove_file(&*socket).unwrap();

        let uds = UnixListener::bind(&*socket).unwrap();
        let stream = UnixListenerStream::new(uds);

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ErrorKinsper::InvalidUri("Invalid database url".to_string()))
            .unwrap();
        let db_context = Database::connect(&database_url).await.unwrap();
        db_context.create_table().await.unwrap();

        let serve_future = async {
            let result = Server::builder()
                .add_service(UserServiceServer::new(MyUserService { db_context }))
                .serve_with_incoming(stream)
                .await;
            // Server must be running fine...
            assert!(result.is_ok());
        };

        let socket = Arc::clone(&socket);
        // Connect to the server over a Unix socket
        // The URL will be ignored.
        let channel = Endpoint::try_from("http://any.url")
            .unwrap()
            .connect_with_connector(service_fn(move |_: Uri| {
                let socket = Arc::clone(&socket);
                async move { UnixStream::connect(&*socket).await }
            }))
            .await
            .unwrap();

        let client = UserServiceClient::new(channel);

        (serve_future, client)
    }

    async fn teardown(mut client: UserServiceClient<Channel>) -> sqlx::Result<()> {
        if TEST_COUNTER.fetch_sub(1, Ordering::SeqCst) == 1 {
            println!("Dropping table!");
            client
                .reset_user_table(Request::new(ResetUserTableRequest {}))
                .await
                .unwrap();
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_01_create_user_valid_is_ok() {
        let (serve_future, mut client) = server_and_client_stub().await;

        let request_future = async {
            let response = client
                .create_user(Request::new(CreateUserRequest {
                    id: Some(UserId {
                        id: "test01_id".to_string(),
                    }),
                    name: "name".to_string(),
                    mail: "mail@mail.com".to_string(),
                }))
                .await;
            assert!(response.is_ok());
            teardown(client).await.unwrap();
        };

        tokio::select! {
            _ = serve_future => panic!("server returned first"),
            _ = request_future => (),
        }
    }

    #[tokio::test]
    async fn test_02_create_user_with_invalid_mail_is_err() {
        let (serve_future, mut client) = server_and_client_stub().await;

        let request_future = async {
            let response = client
                .create_user(Request::new(CreateUserRequest {
                    id: Some(UserId {
                        id: "test02_id".to_string(),
                    }),
                    name: "name".to_string(),
                    mail: "mail".to_string(),
                }))
                .await;
            assert!(response.is_err());
            teardown(client).await.unwrap();
        };

        tokio::select! {
            _ = serve_future => panic!("server returned first"),
            _ = request_future => (),
        }
    }

    #[tokio::test]
    async fn test_03_get_user_with_inevitable_id_is_err() {
        let (serve_future, mut client) = server_and_client_stub().await;

        let request_future = async {
            let response = client
                .get_user(Request::new(GetUserRequest {
                    id: Some(UserId {
                        id: "test03_id".to_string(),
                    }),
                }))
                .await;
            assert!(response.is_err());
            teardown(client).await.unwrap();
        };

        tokio::select! {
            _ = serve_future => panic!("server returned first"),
            _ = request_future => (),
        }
    }

    #[tokio::test]
    async fn test_04_get_user_with_valid_id_is_ok() {
        let (serve_future, mut client) = server_and_client_stub().await;

        let request_future = async {
            let _ = client
                .create_user(Request::new(CreateUserRequest {
                    id: Some(UserId {
                        id: "test04_id".to_string(),
                    }),
                    name: "name".to_string(),
                    mail: "name@name.com".to_string(),
                }))
                .await;

            let response = client
                .get_user(Request::new(GetUserRequest {
                    id: Some(UserId {
                        id: "test04_id".to_string(),
                    }),
                }))
                .await;
            assert!(response.is_ok());
            assert_eq!(response.unwrap().into_inner().name, "name");
            teardown(client).await.unwrap();
        };

        tokio::select! {
            _ = serve_future => panic!("server returned first"),
            _ = request_future => (),
        }
    }

    #[tokio::test]
    async fn test_05_get_all_users_with_valid_limit_is_ok() {
        let (serve_future, mut client) = server_and_client_stub().await;

        let request_future = async {
            let _ = client
                .create_user(Request::new(CreateUserRequest {
                    id: Some(UserId {
                        id: "test05_id1".to_string(),
                    }),
                    name: "name".to_string(),
                    mail: "name@name.com".to_string(),
                }))
                .await;

            let _ = client
                .create_user(Request::new(CreateUserRequest {
                    id: Some(UserId {
                        id: "test05_id2".to_string(),
                    }),
                    name: "name2".to_string(),
                    mail: "name2@name.com".to_string(),
                }))
                .await;

            let response = client
                .get_all_users(Request::new(GetAllUserRequest { limit: 2 }))
                .await;
            assert!(response.is_ok());

            teardown(client).await.unwrap();
        };

        tokio::select! {
            _ = serve_future => panic!("server returned first"),
            _ = request_future => (),
        }
    }

    #[tokio::test]
    async fn test_06_update_user_name_with_valid_id_is_ok() {
        let (serve_future, mut client) = server_and_client_stub().await;

        let request_future = async {
            let _ = client
                .create_user(Request::new(CreateUserRequest {
                    id: Some(UserId {
                        id: "test06_id".to_string(),
                    }),
                    name: "name".to_string(),
                    mail: "name@name.com".to_string(),
                }))
                .await;

            let response = client
                .update_name_user(Request::new(UpdateUserNameRequest {
                    id: Some(UserId {
                        id: "test06_id".to_string(),
                    }),
                    name: "name_updated".to_string(),
                }))
                .await;

            assert!(response.is_ok());
            let response_get = client
                .get_user(Request::new(GetUserRequest {
                    id: Some(UserId {
                        id: "test06_id".to_string(),
                    }),
                }))
                .await;
            assert!(response_get.is_ok());
            assert_eq!(response_get.unwrap().into_inner().name, "name_updated");

            teardown(client).await.unwrap();
        };

        tokio::select! {
            _ = serve_future => panic!("server returned first"),
            _ = request_future => (),
        }
    }

    #[tokio::test]
    async fn test_07_update_user_mail_with_valid_id_is_ok() {
        let (serve_future, mut client) = server_and_client_stub().await;

        let request_future = async {
            let _ = client
                .create_user(Request::new(CreateUserRequest {
                    id: Some(UserId {
                        id: "test07_id".to_string(),
                    }),
                    name: "name".to_string(),
                    mail: "name@name.com".to_string(),
                }))
                .await;

            let response = client
                .update_mail_user(Request::new(UpdateUserMailRequest {
                    id: Some(UserId {
                        id: "test07_id".to_string(),
                    }),
                    mail: "mailnew@mail.com".to_string(),
                }))
                .await;

            let response_get = client
                .get_user(Request::new(GetUserRequest {
                    id: Some(UserId {
                        id: "test07_id".to_string(),
                    }),
                }))
                .await;

            assert!(response.is_ok());
            assert!(response_get.is_ok());
            assert_eq!(response_get.unwrap().into_inner().mail, "mailnew@mail.com");
            teardown(client).await.unwrap();
        };

        tokio::select! {
            _ = serve_future => panic!("server returned first"),
            _ = request_future => (),
        }
    }
}

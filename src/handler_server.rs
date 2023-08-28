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
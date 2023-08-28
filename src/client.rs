use clap::Parser;
use kinsper_rust_test::{errors::ErrorKinsper, SERVER_LOCALHOST, SERVER_LOCALPORT};
use tonic::transport::Channel;
use user_service::{
    user_service_client::UserServiceClient, CreateUserRequest, DeleteUserRequest,
    GetAllUserRequest, GetUserRequest, ResetUserTableRequest, UpdateUserMailRequest,
    UpdateUserNameRequest,
};

use kinsper_rust_test::QUERY_LIMIT_CLIENT;
pub mod user_service {
    // Note: The token passed to the include_proto macro (in our case "routeguide") is
    // the name of the package declared in our .proto file, not a filename, e.g "routeguide.rs".
    tonic::include_proto!("user_service");
}

#[derive(Debug, Parser)]
struct Options {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Get(GetOptions),
    GetAll(GetAllOptions),
    Create(CreateOptions),
    Delete(DeleteOptions),
    UpdateName(UpdateNameOptions),
    UpdateMail(UpdateMailOptions),
    ResetTable,
}

async fn reset_table(mut client: UserServiceClient<Channel>) -> Result<(), ErrorKinsper> {
    let request = tonic::Request::new(ResetUserTableRequest {});

    let response = client.reset_user_table(request).await;
    match response {
        Ok(_) => {
            println!("User table reset successfully");
        }
        Err(e) => {
            eprint!("USER TABLE NOT RESET. ERROR: {:?}", e);
        }
    }
    Ok(())
}
#[derive(Debug, Parser)]
struct UpdateNameOptions {
    #[clap(long)]
    id: String,
    #[clap(long)]
    name: String,
}

async fn update_name(
    opts: UpdateNameOptions,
    mut client: UserServiceClient<Channel>,
) -> Result<(), ErrorKinsper> {
    let request = tonic::Request::new(UpdateUserNameRequest {
        id: Some(user_service::UserId { id: opts.id }),
        name: opts.name,
    });

    let response = client.update_name_user(request).await;
    match response {
        Ok(_) => {
            println!("User name updated successfully");
        }
        Err(e) => {
            eprint!("USER NAME NOT UPDATED. ERROR: {:?}", e);
        }
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct UpdateMailOptions {
    #[clap(long)]
    id: String,
    #[clap(long)]
    mail: String,
}

async fn update_mail(
    opts: UpdateMailOptions,
    mut client: UserServiceClient<Channel>,
) -> Result<(), ErrorKinsper> {
    let request = tonic::Request::new(UpdateUserMailRequest {
        id: Some(user_service::UserId { id: opts.id }),
        mail: opts.mail,
    });

    let response = client.update_mail_user(request).await;
    match response {
        Ok(_) => {
            println!("User mail updated successfully");
        }
        Err(e) => {
            eprint!("USER MAIL NOT UPDATED. ERROR: {:?}", e);
        }
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct DeleteOptions {
    #[clap(long)]
    id: String,
}

async fn delete(
    opts: DeleteOptions,
    mut client: UserServiceClient<Channel>,
) -> Result<(), ErrorKinsper> {
    let request = tonic::Request::new(DeleteUserRequest {
        id: Some(user_service::UserId { id: opts.id }),
    });

    let response = client.delete_user(request).await;
    match response {
        Ok(_) => {
            println!("User deleted successfully");
        }
        Err(e) => {
            eprint!("USER NOT DELETED. ERROR: {:?}", e);
        }
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct CreateOptions {
    #[clap(long)]
    id: String,
    #[clap(long)]
    name: String,
    #[clap(long)]
    mail: String,
}

async fn create(
    opts: CreateOptions,
    mut client: UserServiceClient<Channel>,
) -> Result<(), ErrorKinsper> {
    let request = tonic::Request::new(CreateUserRequest {
        id: Some(user_service::UserId { id: opts.id }),
        name: opts.name,
        mail: opts.mail,
    });

    let response = client.create_user(request).await;
    match response {
        Ok(_) => {
            println!("User created successfully");
        }
        Err(e) => {
            eprint!("USER NOT CREATED. ERROR: {:?}", e);
        }
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct GetAllOptions {
    #[clap(default_value = QUERY_LIMIT_CLIENT, long)]
    limit: u32,
}

async fn get_all(
    opts: GetAllOptions,
    mut client: UserServiceClient<Channel>,
) -> Result<(), ErrorKinsper> {
    let request = tonic::Request::new(GetAllUserRequest { limit: opts.limit });

    match client.get_all_users(request).await {
        Ok(response) => {
            let mut stream = response.into_inner();
            while let Ok(user) = stream.message().await {
                if let Some(user) = user {
                    println!(
                        "User obtained - ID: {} | NAME: {} | MAIL: {}",
                        user.id.unwrap().id,
                        user.name,
                        user.mail
                    );
                } else {
                    break;
                }
            }
        }
        Err(e) => {
            if e.code() == tonic::Code::NotFound {
                println!("No users found in the database");
            } else {
                eprint!("ERROR: {:?}", e);
            }
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
struct GetOptions {
    #[clap(long)]
    id: String,
}

async fn get(opts: GetOptions, mut client: UserServiceClient<Channel>) -> Result<(), ErrorKinsper> {
    let request = tonic::Request::new(GetUserRequest {
        id: Some(user_service::UserId { id: opts.id }),
    });

    let response = client.get_user(request).await;
    match response {
        Ok(response) => {
            let response = response.into_inner();
            println!(
                "User obtained - ID: {} | NAME: {} | MAIL: {}",
                response.id.unwrap().id,
                response.name,
                response.mail
            );
        }
        Err(e) => {
            eprint!("USER NOT FOUND. ERROR: {:?}", e);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ErrorKinsper> {
    let opts = Options::parse();

    let addr = format!("http://{}:{}", SERVER_LOCALHOST, SERVER_LOCALPORT);
    let client = UserServiceClient::connect(addr)
        .await
        .map_err(|_| ErrorKinsper::InternalServer("Error connecting to server".to_string()))?;

    use Command::*;
    match opts.command {
        Get(opts) => get(opts, client).await?,
        GetAll(opts) => get_all(opts, client).await?,
        Create(opts) => create(opts, client).await?,
        Delete(opts) => delete(opts, client).await?,
        UpdateName(opts) => update_name(opts, client).await?,
        UpdateMail(opts) => update_mail(opts, client).await?,
        ResetTable => reset_table(client).await?,
    };

    Ok(())
}

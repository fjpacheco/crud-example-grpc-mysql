use clap::Parser;
use kinsper_rust_test::{SERVER_LOCALHOST, SERVER_LOCALPORT};
use user_service::{
    user_service_client::UserServiceClient, CreateUserRequest, DeleteUserRequest,
    GetAllUserRequest, GetUserRequest, UpdateUserMailRequest, UpdateUserNameRequest,
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
}

#[derive(Debug, Parser)]
struct UpdateNameOptions {
    #[clap(long)]
    id: String,
    #[clap(long)]
    name: String,
}

async fn update_name(opts: UpdateNameOptions) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://{}:{}", SERVER_LOCALHOST, SERVER_LOCALPORT);
    let mut client = UserServiceClient::connect(addr).await?;

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

async fn update_mail(opts: UpdateMailOptions) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://{}:{}", SERVER_LOCALHOST, SERVER_LOCALPORT);
    let mut client = UserServiceClient::connect(addr).await?;

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

async fn delete(opts: DeleteOptions) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://{}:{}", SERVER_LOCALHOST, SERVER_LOCALPORT);
    let mut client = UserServiceClient::connect(addr).await?;

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

async fn create(opts: CreateOptions) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://{}:{}", SERVER_LOCALHOST, SERVER_LOCALPORT);
    let mut client = UserServiceClient::connect(addr).await?;

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

async fn get_all(opts: GetAllOptions) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://{}:{}", SERVER_LOCALHOST, SERVER_LOCALPORT);
    let mut client = UserServiceClient::connect(addr).await?;

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

async fn get(opts: GetOptions) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://{}:{}", SERVER_LOCALHOST, SERVER_LOCALPORT);
    let mut client = UserServiceClient::connect(addr).await?;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::parse();

    use Command::*;
    match opts.command {
        Get(opts) => get(opts).await?,
        GetAll(opts) => get_all(opts).await?,
        Create(opts) => create(opts).await?,
        Delete(opts) => delete(opts).await?,
        UpdateName(opts) => update_name(opts).await?,
        UpdateMail(opts) => update_mail(opts).await?,
    };

    Ok(())
}

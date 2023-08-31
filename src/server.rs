use dotenv::dotenv;
use kinsper_rust_test::data::context::Database;
use kinsper_rust_test::errors::ErrorKinsper;
use kinsper_rust_test::handler_server::user_service::user_service_server::UserServiceServer;
use kinsper_rust_test::handler_server::MyUserService;
use kinsper_rust_test::{initialize_logging, MAX_T_RUNTIME_SERVER, SERVER_LOCALHOST, SERVER_LOCALPORT};
use tonic::transport::Server;

fn main() -> Result<(), ErrorKinsper> {
    dotenv().ok();
    initialize_logging();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(MAX_T_RUNTIME_SERVER)
        .thread_name("server-runtime")
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let database_url = std::env::var("DATABASE_URL")
                .map_err(|_| ErrorKinsper::InvalidUri("Invalid database url".to_string()))?;
            let db_context = Database::connect(&database_url).await?;
            db_context.create_table().await?;

            let addr = format!("{}:{}", SERVER_LOCALHOST, SERVER_LOCALPORT)
                .parse()
                .map_err(|_| ErrorKinsper::InvalidUri("Invalid server url".to_string()))?;
            let user_service = MyUserService { db_context };
            log::info!("Listening on {}", addr);

            Server::builder()
                .add_service(UserServiceServer::new(user_service))
                .serve(addr)
                .await
                .map_err(|err| {
                    ErrorKinsper::InternalServer(format!("Server error initializing: {}", err))
                })?;
            Ok(())
        })
}

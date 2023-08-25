use dotenv::dotenv;
use kinsper_rust_test::data::model::UserModel;
use log::LevelFilter;
use std::env;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

    
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_context = kinsper_rust_test::data::context::Database::new(&database_url).await.unwrap();
    
    db_context.users.drop_table().await.unwrap();
    db_context.users.create_table().await.unwrap();

    let users = vec![
        UserModel {
            id: "12".to_string(),
            name: "Fede".to_string(),
            mail: "fede@test.com".to_string(),
        },
        UserModel {
            id: "23".to_string(),
            name: "Abel".to_string(),
            mail: "abel@test.com".to_string(),
        },
    ];

    for user in users {
        db_context.users.add_user(&user).await.unwrap();
    }

    let selected_users =  db_context.users.get_users(None).await.unwrap();

    println!("selected_users: {:?}", selected_users);

    Ok(())
}

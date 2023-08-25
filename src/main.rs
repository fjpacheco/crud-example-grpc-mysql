use dotenv::dotenv;
use kinsper_rust_test::model::UserModel;
use log::LevelFilter;
use sqlx::mysql::MySqlPoolOptions;
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
    let pool = match MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            log::info!("🚀 Connected to the database");
            pool
        }
        Err(err) => {
            log::error!("❌ Couldn't connect to the database: {}", err);
            std::process::exit(1);
        }
    };
    let users = vec![
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
    ];

    for user in users {
        sqlx::query(r"INSERT INTO users (id, name, mail) VALUES (?, ?, ?)")
            .bind(user.id)
            .bind(user.name)
            .bind(user.mail)
            .execute(&pool)
            .await?;
    }

    let selected_users = sqlx::query_as!(UserModel, r#"SELECT * FROM users"#)
        .fetch_all(&pool)
        .await?;

    println!("selected_users: {:?}", selected_users);

    Ok(())
}

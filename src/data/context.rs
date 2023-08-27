use std::sync::Arc;

use sqlx::MySqlPool;

use crate::errors::ErrorKinsper;

pub struct Database {
    // https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#why-use-a-pool
    pub pool: Arc<MySqlPool>,
}

fn debug_pool_db(pool: Arc<MySqlPool>) {
    if std::env::var("RUST_LOG")
        .unwrap_or_default()
        .contains("debug")
    {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(300));
            loop {
                interval.tick().await;
                log::debug!("ARC COUNT: {:?}", Arc::strong_count(&pool));
                log::debug!("POOL: {:?}", pool);
            }
        });
    }
}

impl Database {
    pub async fn connect(sql_url: &str) -> Result<Database, ErrorKinsper> {
        let connection = MySqlPool::connect(sql_url).await.map_err(|err| {
            ErrorKinsper::new(
                crate::errors::TypeErrorKinsper::ConnectionError,
                format!("Couldn't connect to the database: {}", err),
            )
        })?;
        let pool = Arc::new(connection);

        debug_pool_db(pool.clone());

        Ok(Database { pool })
    }
}

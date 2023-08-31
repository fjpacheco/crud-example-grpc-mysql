use std::sync::Arc;

use sqlx::{mysql::MySqlPoolOptions, pool::PoolOptions, MySqlPool};

use crate::{errors::ErrorKinsper, MAX_CONNECTIONS_DB_POOL};

pub struct Database {
    // https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#why-use-a-pool
    pub pool: Arc<MySqlPool>,
}

impl Database {
    pub async fn connect(sql_url: &str) -> Result<Database, ErrorKinsper> {
        let pool_opt = MySqlPoolOptions::new()
            .max_connections(MAX_CONNECTIONS_DB_POOL)
            .connect(sql_url)
            .await
            .map_err(|err| {
                ErrorKinsper::ConnectionError(format!("Couldn't connect to the database: {}", err))
            })?;
        // let connection = MySqlPool::connect(sql_url).await.map_err(|err| {
        //     ErrorKinsper::ConnectionError(format!("Couldn't connect to the database: {}", err))
        // })?;
        log::warn!("{:?}", pool_opt.options());
        let pool = Arc::new(pool_opt);

        Ok(Database { pool })
    }
}

use std::sync::Arc;

use sqlx::MySqlPool;

use crate::errors::ErrorKinsper;

pub struct Database {
    // https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#why-use-a-pool
    pub pool: Arc<MySqlPool>,
}

impl Database {
    pub async fn connect(sql_url: &str) -> Result<Database, ErrorKinsper> {
        let connection = MySqlPool::connect(sql_url).await.map_err(|err| {
            ErrorKinsper::ConnectionError(format!("Couldn't connect to the database: {}", err))
        })?;
        let pool = Arc::new(connection);

        Ok(Database { pool })
    }
}

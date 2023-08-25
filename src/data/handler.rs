use crate::errors::ErrorKinsper;

use super::{context::Table, model::UserModel};


impl<'c> Table<'c, UserModel> {
    pub async fn drop_table(&self) -> Result<(), ErrorKinsper> {
        sqlx::query("DROP TABLE IF EXISTS users;")
            .execute(&*self.pool)
            .await
            .map_err(|err| {
                ErrorKinsper::new(
                    crate::errors::TypeErrorKinsper::MySqlError,
                    format!("Couldn't drop the table: {}", err),
                )
            })?;

        Ok(())
    }

    pub async fn create_table(&self) -> Result<(), ErrorKinsper> {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS users (
                id VARCHAR(48) PRIMARY KEY NOT NULL,
                name VARCHAR(256) NOT NULL,
                mail VARCHAR(256) NOT NULL
                )"#,
            )
        .execute(&*self.pool)
        .await
        .map_err(|err| {
            ErrorKinsper::new(
                crate::errors::TypeErrorKinsper::MySqlError,
                format!("Couldn't create the table: {}", err),
            )
        })?;

        Ok(())
    }

}
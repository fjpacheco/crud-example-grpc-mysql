use crate::{data::QUERY_LIMIT, errors::ErrorKinsper};

use super::{context::Table, model::UserModel};

impl<'c> Table<'c, UserModel> {
    pub async fn drop_table(&self) -> Result<(), ErrorKinsper> {
        sqlx::query("DROP TABLE IF EXISTS users;")
            .execute(&*self.pool)
            .await?;

        log::info!("Table users dropped.");
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
        .await?;

        log::info!("Table users created.");
        Ok(())
    }

    pub async fn add_user(&self, user: &UserModel) -> Result<u64, ErrorKinsper> {
        let result = sqlx::query(
            r#"
            INSERT INTO users (`id`, `name`, `mail`)
            VALUES(?, ?, ?)"#,
        )
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.mail)
        .execute(&*self.pool)
        .await?;

        log::info!(
            "Rows affected: {}. New user added (ID-{}).",
            result.rows_affected(),
            user.id
        );
        Ok(result.rows_affected())
    }

    pub async fn get_users(&self, limit: Option<u32>) -> Result<Vec<UserModel>, ErrorKinsper> {
        let result = sqlx::query_as::<_, UserModel>(
            r#"
                SELECT * 
                FROM users 
                LIMIT ?"#,
        )
        .bind(limit.unwrap_or(QUERY_LIMIT))
        .fetch_all(&*self.pool)
        .await?;

        log::info!("Rows selected: {}.", result.len());
        Ok(result)
    }
}

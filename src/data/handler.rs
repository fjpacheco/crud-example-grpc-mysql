use crate::{data::QUERY_LIMIT, errors::ErrorKinsper};

use super::{
    context::Table,
    model::UserModel,
    scheme::{CreateUserScheme, UpdateUserSchema},
};

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

    pub async fn add_user(&self, user: &CreateUserScheme) -> Result<u64, ErrorKinsper> {
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

    pub async fn get_user_by_id(&self, id: &str) -> Result<UserModel, ErrorKinsper> {
        let result = sqlx::query_as::<_, UserModel>(
            r#"
                SELECT * 
                FROM users 
                WHERE id = ?"#,
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await?;

        log::info!("User selected (ID-{}).", id);
        Ok(result)
    }

    pub async fn update_user(
        &self,
        id: String,
        user: UpdateUserSchema,
    ) -> Result<u64, ErrorKinsper> {
        let result = sqlx::query(
            format!(
                r#"
                UPDATE users 
                SET {} 
                WHERE id = ?"#,
                user.query_set()
            )
            .as_str(),
        )
        .bind(&id)
        .execute(&*self.pool)
        .await?;

        log::info!(
            "Rows affected: {}. User updated (ID-{}).",
            result.rows_affected(),
            id
        );
        Ok(result.rows_affected())
    }

    pub async fn delete_user(&self, id: &str) -> Result<u64, ErrorKinsper> {
        let result = sqlx::query(
            r#"
            DELETE FROM users 
            WHERE id = ?"#,
        )
        .bind(id)
        .execute(&*self.pool)
        .await?;

        log::info!(
            "Rows affected: {}. User deleted (ID-{}).",
            result.rows_affected(),
            id
        );
        Ok(result.rows_affected())
    }
}


// tests

#[cfg(test)]
mod handler_tests {
    use sqlx::{Pool, MySqlPool, pool::PoolConnection};
    use dotenv::dotenv;

    use crate::data::context::Database;

    #[sqlx::test]
    async fn test_get_user_inexistent() -> sqlx::Result<()> {
        // CONTEXT TEST!
        dotenv().ok();  
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_context = Database::new(&database_url)
            .await
            .unwrap();
        db_context.users.drop_table().await.unwrap();
        db_context.users.create_table().await.unwrap();

        let user_inserted = db_context.users.get_user_by_id("12").await;
        assert!(user_inserted.is_err());
        Ok(())
    }

    #[sqlx::test]
    async fn test_add_user_and_get_ready() -> sqlx::Result<()> {
        // CONTEXT TEST!
        dotenv().ok();  
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_context = Database::new(&database_url)
            .await
            .unwrap();
        db_context.users.drop_table().await.unwrap();
        db_context.users.create_table().await.unwrap();

        db_context.users.add_user(&crate::data::scheme::CreateUserScheme {
            id: "15".to_string(),
            name: "Fede".to_string(),
            mail: "fede@gmail.com".to_string()}).await.unwrap();

        let user_inserted = db_context.users.get_user_by_id("15").await.unwrap();

        assert_eq!(user_inserted.id, "15".to_string());
        Ok(())
    }


}
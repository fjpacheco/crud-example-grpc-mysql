use crate::{data::QUERY_LIMIT, errors::ErrorKinsper};

use super::{
    context::Database,
    model::UserModel,
    scheme::{CreateUserScheme, UpdateUserSchema},
};

impl Database {
    fn debug_thread(&self) {
        log::debug!(
            "[CURRENT_THREAD: {:?}] | [THREAD_NAME: {:?}]",
            std::thread::current().id(),
            std::thread::current().name().unwrap()
        );
    }
    pub async fn drop_table(&self) -> Result<(), ErrorKinsper> {
        self.debug_thread();

        sqlx::query("DROP TABLE IF EXISTS users;")
            .execute(self.pool.clone().as_ref())
            .await?;

        Ok(())
    }

    pub async fn create_table(&self) -> Result<(), ErrorKinsper> {
        self.debug_thread();

        sqlx::query(
            r#"
                CREATE TABLE IF NOT EXISTS users (
                id VARCHAR(48) PRIMARY KEY NOT NULL,
                name VARCHAR(256) NOT NULL,
                mail VARCHAR(256) NOT NULL
                )"#,
        )
        .execute(self.pool.clone().as_ref())
        .await?;

        Ok(())
    }

    pub async fn reset_table(&self) -> Result<(), ErrorKinsper> {
        self.debug_thread();

        self.drop_table().await?;
        self.create_table().await?;

        Ok(())
    }

    pub async fn add_user(&self, user: &CreateUserScheme) -> Result<u64, ErrorKinsper> {
        self.debug_thread();

        let result = sqlx::query(
            r#"
            INSERT INTO users (`id`, `name`, `mail`)
            VALUES(?, ?, ?)"#,
        )
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.mail)
        .execute(self.pool.clone().as_ref())
        .await?;

        match result.rows_affected() {
            0 => Err(ErrorKinsper::AlreadyExists(
                "User already exists.".to_string(),
            )),
            _ => Ok(result.rows_affected()),
        }
    }

    pub async fn get_users(&self, limit: Option<u32>) -> Result<Vec<UserModel>, ErrorKinsper> {
        self.debug_thread();

        let result = sqlx::query_as::<_, UserModel>(
            r#"
                SELECT * 
                FROM users 
                LIMIT ?"#,
        )
        .bind(limit.unwrap_or(QUERY_LIMIT))
        .fetch_all(self.pool.clone().as_ref())
        .await?;

        if result.is_empty() {
            Err(ErrorKinsper::NotFound("No users found.".to_string()))
        } else {
            Ok(result)
        }
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<UserModel, ErrorKinsper> {
        self.debug_thread();

        let result = sqlx::query_as::<_, UserModel>(
            r#"
                SELECT * 
                FROM users 
                WHERE id = ?"#,
        )
        .bind(id)
        .fetch_one(self.pool.clone().as_ref())
        .await?;

        Ok(result)
    }

    pub async fn update_user(
        &self,
        id: &str,
        user: &UpdateUserSchema,
    ) -> Result<u64, ErrorKinsper> {
        self.debug_thread();

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
        .bind(id)
        .execute(self.pool.clone().as_ref())
        .await?;

        match result.rows_affected() {
            0 => Err(ErrorKinsper::NotFound("User not found.".to_string())),
            _ => Ok(result.rows_affected()),
        }
    }

    pub async fn delete_user(&self, id: &str) -> Result<u64, ErrorKinsper> {
        self.debug_thread();

        let result = sqlx::query(
            r#"
            DELETE FROM users 
            WHERE id = ?"#,
        )
        .bind(id)
        .execute(self.pool.clone().as_ref())
        .await?;

        match result.rows_affected() {
            0 => Err(ErrorKinsper::NotFound("User not found.".to_string())),
            _ => Ok(result.rows_affected()),
        }
    }
}

// tests

#[cfg(test)]
mod handler_tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use dotenv::dotenv;

    use crate::data::{
        context::Database,
        scheme::{CreateUserScheme, UpdateUserSchema},
    };

    // Este mecanismo es para que se limpie la tabla al final de todos los tests
    // Como rust ejecuta los tests en paralelo (y yo quiero aprovechar eso) entonces
    // necesito que solo 1 test se encargue de limpiar la tabla
    // Se valida el comportamiento viendo el print "cargo test -- --show-output"
    // TODO: evitar usar println! y usar log::info! o log::debug!

    const NUMBER_TESTS: usize = 7;
    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(NUMBER_TESTS);
    async fn setup() -> sqlx::Result<Arc<Database>> {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_context = Arc::new(Database::connect(&database_url).await.unwrap());
        db_context.create_table().await.unwrap();
        Ok(db_context)
    }

    async fn teardown(db_context: Arc<Database>) -> sqlx::Result<()> {
        if TEST_COUNTER.fetch_sub(1, Ordering::SeqCst) == 1 {
            println!("Dropping table!");
            db_context.drop_table().await.unwrap();
        }
        Ok(())
    }

    #[tokio::test]
    async fn test01_when_get_user_by_id_given_inexistent_id_then_returns_error() -> sqlx::Result<()>
    {
        let db_context = setup().await?;

        let user_inserted = db_context.get_user_by_id("12").await;

        assert!(user_inserted.is_err());
        teardown(db_context).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test02_when_add_user_given_valid_user_then_can_get_that_user() -> sqlx::Result<()> {
        let db_context = setup().await?;
        let new_user = crate::data::scheme::CreateUserScheme {
            id: "15".to_string(),
            name: "Fede".to_string(),
            mail: "fede@gmail.com".to_string(),
        };
        db_context.add_user(&new_user).await.unwrap();

        let user_inserted = db_context.get_user_by_id("15").await.unwrap();

        assert_eq!(user_inserted.id, "15".to_string());
        teardown(db_context).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test03_when_get_users_given_limit_then_returns_limited_number_of_users(
    ) -> sqlx::Result<()> {
        let db_context = setup().await?;

        let new_users = vec![
            CreateUserScheme {
                id: "20".to_string(),
                name: "User 1".to_string(),
                mail: "user1@example.com".to_string(),
            },
            CreateUserScheme {
                id: "21".to_string(),
                name: "User 2".to_string(),
                mail: "user2@example.com".to_string(),
            },
            CreateUserScheme {
                id: "23".to_string(),
                name: "User 3".to_string(),
                mail: "user3@example.com".to_string(),
            },
        ];

        for user in new_users {
            db_context.add_user(&user).await.unwrap();
        }

        let users = db_context.get_users(Some(2)).await.unwrap();

        assert_eq!(users.len(), 2);
        teardown(db_context).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test04_when_update_user_given_valid_id_and_schema_then_updated_successfully(
    ) -> sqlx::Result<()> {
        let db_context = setup().await?;

        let new_user = CreateUserScheme {
            id: "9494".to_string(),
            name: "Jorge".to_string(),
            mail: "jorge@gmail.com".to_string(),
        };
        db_context.add_user(&new_user).await.unwrap();

        let updated_user = UpdateUserSchema::new()
            .with_name("Jorge Updated".to_string())
            .with_mail("jorge_updated@gmail.com".to_string())
            .finalize()
            .unwrap();

        db_context.update_user("9494", &updated_user).await.unwrap();

        let user_updated = db_context.get_user_by_id("9494").await.unwrap();

        assert_eq!(user_updated.name, updated_user.name.unwrap());
        assert_eq!(user_updated.mail, updated_user.mail.unwrap());

        teardown(db_context).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test05_when_update_user_ineexistent_id_then_returns_error() -> sqlx::Result<()> {
        let db_context = setup().await?;

        let updated_user = UpdateUserSchema::new()
            .with_name("Jorge Updated".to_string())
            .with_mail("jorgito@gmail.com".to_string())
            .finalize()
            .unwrap();

        let result = db_context.update_user("9491", &updated_user).await;

        assert!(result.is_err());
        teardown(db_context).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test06_when_delete_user_given_inexistent_id_then_returns_error() -> sqlx::Result<()> {
        let db_context = setup().await?;

        let result = db_context.delete_user("9492").await;

        assert!(result.is_err());
        teardown(db_context).await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test07_when_delete_user_given_valid_id_then_deleted_successfully() -> sqlx::Result<()>
    {
        let db_context = setup().await?;

        let new_user = CreateUserScheme {
            id: "25".to_string(),
            name: "Luis".to_string(),
            mail: "luis@gmail.com".to_string(),
        };
        db_context.add_user(&new_user).await.unwrap();

        db_context.delete_user("25").await.unwrap();

        let deleted_user = db_context.get_user_by_id("25").await;

        assert!(deleted_user.is_err());
        teardown(db_context).await.unwrap();
        Ok(())
    }
}

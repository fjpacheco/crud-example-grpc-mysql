use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, MySqlPool};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::errors::ErrorKinsper;

use super::model::UserModel;

pub struct Table<'c, T>
where
    T: FromRow<'c, MySqlRow>,
{
    pub pool: Arc<MySqlPool>,
    _from_row: fn(&'c MySqlRow) -> Result<T, sqlx::Error>,
    _marker: PhantomData<&'c T>,
}

impl<'c, T> Table<'c, T>
where
    T: FromRow<'c, MySqlRow>,
{
    fn new(pool: Arc<MySqlPool>) -> Self {
        Table {
            pool,
            _from_row: T::from_row,
            _marker: PhantomData,
        }
    }
}

pub struct Database<'c> {
    pub users: Arc<Table<'c, UserModel>>,
}

impl<'a> Database<'a> {
    pub async fn new(sql_url: &str) -> Result<Database<'a>, ErrorKinsper> {
        let connection = MySqlPool::connect(sql_url).await.map_err(|err| {
            ErrorKinsper::new(
                crate::errors::TypeErrorKinsper::ConnectionError,
                format!("Couldn't connect to the database: {}", err),
            )
        })?;
        let pool = Arc::new(connection);

        Ok(Database {
            users: Arc::from(Table::new(pool.clone())),
        })
    }
}
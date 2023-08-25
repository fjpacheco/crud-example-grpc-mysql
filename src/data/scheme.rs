use serde::{Deserialize, Serialize};

use crate::errors::ErrorKinsper;

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct CreateUserScheme {
    pub id: String,
    pub name: String,
    pub mail: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateUserSchema {
    pub id: Option<String>,
    pub name: Option<String>,
    pub mail: Option<String>,
    pub query_set: String,
}

impl UpdateUserSchema {
    pub fn new(
        id: Option<String>,
        name: Option<String>,
        mail: Option<String>,
    ) -> Result<UpdateUserSchema, ErrorKinsper> {
        let mut query_set_vec = Vec::new();
        if let Some(id) = id.as_ref() {
            query_set_vec.push(format!("id = '{}'", id));
        }

        if let Some(name) = name.as_ref() {
            query_set_vec.push(format!("name = '{}'", name));
        }

        if let Some(mail) = mail.as_ref() {
            query_set_vec.push(format!("mail = '{}'", mail));
        }

        if query_set_vec.is_empty() {
            return Err(ErrorKinsper::new(
                crate::errors::TypeErrorKinsper::UpdateSchemeError,
                "No fields to update.".to_string(),
            ));
        }

        let query_set = query_set_vec.join(", ");

        Ok(UpdateUserSchema {
            id,
            name,
            mail,
            query_set,
        })
    }

    pub fn query_set(&self) -> &String {
        &self.query_set
    }
}

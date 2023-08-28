use serde::{Deserialize, Serialize};

use crate::errors::ErrorKinsper;

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct CreateUserScheme {
    pub id: String,
    pub name: String,
    pub mail: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateUserSchema {
    pub id: Option<String>,
    pub name: Option<String>,
    pub mail: Option<String>,
    pub query_set: String,
}

impl UpdateUserSchema {
    pub fn new() -> Self {
        UpdateUserSchema {
            id: None,
            name: None,
            mail: None,
            query_set: String::new(),
        }
    }

    pub fn query_set(&self) -> &String {
        &self.query_set
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_mail(mut self, mail: String) -> Self {
        self.mail = Some(mail);
        self
    }

    pub fn finalize(self) -> Result<Self, ErrorKinsper> {
        let query_set = self.prepare_query_set()?;
        Ok(UpdateUserSchema { query_set, ..self })
    }

    fn prepare_query_set(&self) -> Result<String, ErrorKinsper> {
        let updates: Vec<String> = vec![
            self.id.as_ref().map(|id| format!("id = '{}'", id)),
            self.name.as_ref().map(|name| format!("name = '{}'", name)),
            self.mail.as_ref().map(|mail| format!("mail = '{}'", mail)),
        ]
        .into_iter()
        .flatten()
        .collect();

        if updates.is_empty() {
            return Err(ErrorKinsper::UpdateSchemeError(
                "No fields to update.".to_string(),
            ));
        }

        Ok(updates.join(", "))
    }
}

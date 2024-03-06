use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::entity::user::User;

#[derive(Clone, Debug)]
pub(crate) struct DatabaseUser(Arc<HashMap<Uuid, User>>);

impl AsRef<HashMap<Uuid, User>> for DatabaseUser {
    fn as_ref(&self) -> &HashMap<Uuid, User> {
        &self.0
    }
}

impl From<Vec<User>> for DatabaseUser {
    fn from(value: Vec<User>) -> Self {
        Self(Arc::new(HashMap::from_iter(
            value.into_iter().map(|item| (item.id, item)),
        )))
    }
}

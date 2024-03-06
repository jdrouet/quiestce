use uuid::Uuid;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

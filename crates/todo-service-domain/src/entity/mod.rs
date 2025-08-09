#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TodoEntity {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub is_done: bool,
    pub updated_at: time::OffsetDateTime,
}

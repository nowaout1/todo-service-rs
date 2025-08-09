use todo_service_domain::entity;

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct TodoDto {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub is_done: bool,
    pub updated_at: time::OffsetDateTime,
}

impl From<TodoDto> for entity::TodoEntity {
    fn from(dto: TodoDto) -> Self {
        Self {
            id: dto.id,
            title: dto.title,
            description: dto.description,
            is_done: dto.is_done,
            updated_at: dto.updated_at,
        }
    }
}

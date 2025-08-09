use async_trait::async_trait;

use crate::entity;

pub mod constants;

#[async_trait]
pub trait FetchTodoById {
    type TodoDto: Into<entity::TodoEntity>;
    type Error: std::fmt::Debug;

    async fn fetch_todo_by_id(
        &self,
        req: FetchTodoByIdRequest,
    ) -> Result<Option<Self::TodoDto>, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FetchTodoByIdRequest {
    pub id: uuid::Uuid,
}

#[async_trait]
pub trait FetchTodosByRange {
    type TodoDto: Into<entity::TodoEntity>;
    type Error: std::fmt::Debug;

    async fn fetch_todos_by_range(
        &self,
        req: FetchTodosByRangeRequest,
    ) -> Result<Vec<Self::TodoDto>, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FetchTodosByRangeRequest {
    pub offset: usize,
    pub limit: usize,
}

#[async_trait]
pub trait CreateTodo {
    type TodoDto: Into<entity::TodoEntity>;
    type Error: std::fmt::Debug;

    async fn create_todo(&self, req: CreateTodoRequest) -> Result<Self::TodoDto, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: String,
}

#[async_trait]
pub trait UpdateTodo {
    type TodoDto: Into<entity::TodoEntity>;
    type Error: std::fmt::Debug;

    async fn update_todo(&self, req: UpdateTodoRequest) -> Result<Self::TodoDto, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateTodoRequest {
    pub id: uuid::Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_done: Option<bool>,
}

#[async_trait]
pub trait DeleteTodo {
    type Error: std::fmt::Debug;

    async fn delete_todo(&self, req: DeleteTodoRequest) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeleteTodoRequest {
    pub id: uuid::Uuid,
}

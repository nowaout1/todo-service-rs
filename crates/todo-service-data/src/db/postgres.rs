use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use todo_service_domain::repository;

use crate::dto;

pub const DEFAULT_POSTGRES_MAX_CONNECTIONS: u32 = 12;

#[derive(Debug, Clone)]
pub struct TodoPostgres<const MAX_DB_CONNECTIONS: u32 = DEFAULT_POSTGRES_MAX_CONNECTIONS> {
    pool: PgPool,
}

impl<const MAX_DB_CONNECTIONS: u32> TodoPostgres<MAX_DB_CONNECTIONS> {
    pub async fn init(db_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(MAX_DB_CONNECTIONS)
            .connect(db_url)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl repository::FetchTodoById for TodoPostgres {
    type TodoDto = dto::TodoDto;
    type Error = sqlx::Error;

    async fn fetch_todo_by_id(
        &self,
        request: repository::FetchTodoByIdRequest,
    ) -> Result<Option<Self::TodoDto>, Self::Error> {
        sqlx::query_as!(
            dto::TodoDto,
            r#"
                SELECT id, title, description, is_done, updated_at
                FROM todos
                WHERE id = $1
                LIMIT 1
            "#,
            request.id
        )
        .fetch_optional(&self.pool)
        .await
    }
}

#[async_trait]
impl repository::FetchTodosByRange for TodoPostgres {
    type TodoDto = dto::TodoDto;
    type Error = sqlx::Error;

    async fn fetch_todos_by_range(
        &self,
        request: repository::FetchTodosByRangeRequest,
    ) -> Result<Vec<Self::TodoDto>, Self::Error> {
        let limit = request.limit.clamp(
            repository::constants::DEFAULT_TODOS_MIN_FETCH,
            repository::constants::DEFAULT_TODOS_MAX_FETCH,
        );

        sqlx::query_as!(
            dto::TodoDto,
            r#"
                SELECT id, title, description, is_done, updated_at
                FROM todos
                LIMIT $1
                OFFSET $2
            "#,
            limit as i32,
            request.offset as i32
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[async_trait]
impl repository::CreateTodo for TodoPostgres {
    type TodoDto = dto::TodoDto;
    type Error = sqlx::Error;

    async fn create_todo(
        &self,
        request: repository::CreateTodoRequest,
    ) -> Result<Self::TodoDto, Self::Error> {
        let id = uuid::Uuid::now_v7();

        let row = sqlx::query!(
            r#"
                    INSERT INTO todos (id, title, description)
                    VALUES ($1, $2, $3)
                    RETURNING updated_at
                "#,
            id,
            request.title,
            request.description
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(dto::TodoDto {
            id,
            title: request.title,
            description: request.description,
            is_done: false,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl repository::UpdateTodo for TodoPostgres {
    type TodoDto = dto::TodoDto;
    type Error = sqlx::Error;

    async fn update_todo(
        &self,
        request: repository::UpdateTodoRequest,
    ) -> Result<Self::TodoDto, Self::Error> {
        let row = sqlx::query!(
            r#"
                UPDATE todos
                SET
                    title = COALESCE($1, title),
                    description = COALESCE($2, description),
                    is_done = COALESCE($3, is_done)
                WHERE id = $4
                RETURNING title, description, is_done, updated_at
            "#,
            request.title,
            request.description,
            request.is_done,
            request.id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(dto::TodoDto {
            id: request.id,
            title: row.title,
            description: row.description,
            is_done: row.is_done,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl repository::DeleteTodo for TodoPostgres {
    type Error = sqlx::Error;

    async fn delete_todo(&self, request: repository::DeleteTodoRequest) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
                DELETE FROM todos
                WHERE id = $1
            "#,
            request.id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

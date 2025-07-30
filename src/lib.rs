// Ensure everything is implemented in 100% Safe Rust.
#![forbid(unsafe_code)]

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tonic::{Request, Response, Status};

use crate::todo::{
    CreateTodoRequest, CreateTodoResponse, DeleteTodoRequest, DeleteTodoResponse,
    FetchManyTodosRequest, FetchManyTodosResponse, FetchOneTodoRequest, FetchOneTodoResponse,
    TodoItem, UpdateTodoRequest, UpdateTodoResponse, todo_server::Todo,
};

pub mod config;
pub mod todo {
    tonic::include_proto!("todo");
}
pub mod utils;

#[derive(Debug, Clone)]
pub struct TodoService<const MAX_POOL_DB_CONNECTIONS: u32 = 5> {
    pool: PgPool,
}

impl<const MAX_POOL_DB_CONNECTIONS: u32> TodoService<MAX_POOL_DB_CONNECTIONS> {
    pub async fn init(db_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(MAX_POOL_DB_CONNECTIONS)
            .connect(db_url)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }
}

#[tonic::async_trait]
impl Todo for TodoService {
    async fn create_todo(
        &self,
        req: Request<CreateTodoRequest>,
    ) -> Result<Response<CreateTodoResponse>, Status> {
        // TODO: Refactor me

        let CreateTodoRequest { title, description } = req.into_inner();

        let id = uuid::Uuid::now_v7();
        let insert_query = sqlx::query!(
            r#"
                INSERT INTO todos (id, title, description)
                VALUES ($1, $2, $3)
                RETURNING updated_at
            "#,
            id,
            title,
            description
        )
        .fetch_one(&self.pool);

        match insert_query.await {
            Ok(row) => {
                let todo = TodoItem {
                    id: id.into(),
                    title,
                    description,
                    is_done: false,
                    updated_at: Some(utils::chrono_offset_date_time_to_prost_timestamp(
                        row.updated_at,
                    )),
                };

                let res = CreateTodoResponse {
                    todo: Some(todo),
                    msg: String::from("Created"),
                };

                Ok(res.into())
            }
            Err(error) => {
                tracing::error!("Database error: {error:?}");

                Err(Status::internal("Failed to create todo"))
            }
        }
    }

    async fn update_todo(
        &self,
        req: Request<UpdateTodoRequest>,
    ) -> Result<Response<UpdateTodoResponse>, Status> {
        // TODO: Refactor me

        let UpdateTodoRequest {
            id,
            title,
            description,
            is_done,
        } = req.into_inner();

        let Ok(id) = uuid::Uuid::try_from(id) else {
            return Err(Status::invalid_argument("Invalid todo ID"));
        };

        if title.is_none() && description.is_none() && is_done.is_none() {
            return Err(Status::invalid_argument(
                "At least one field must be filled in",
            ));
        }

        let update_query = sqlx::query!(
            r#"
                UPDATE todos
                SET
                    title = COALESCE($1, title),
                    description = COALESCE($2, description),
                    is_done = COALESCE($3, is_done)
                WHERE id = $4
                RETURNING title, description, is_done, updated_at
            "#,
            title,
            description,
            is_done,
            id,
        )
        .fetch_optional(&self.pool);

        let Ok(maybe_todo) = update_query.await else {
            return Err(Status::internal("Failed to update todo"));
        };

        let res = match maybe_todo {
            Some(todo) => UpdateTodoResponse {
                todo: Some(TodoItem {
                    id: id.to_string(),
                    title: todo.title,
                    description: todo.description,
                    is_done: todo.is_done,
                    updated_at: Some(utils::chrono_offset_date_time_to_prost_timestamp(
                        todo.updated_at,
                    )),
                }),
                msg: String::from("Todo was updated"),
            },
            None => UpdateTodoResponse {
                todo: None,
                msg: String::from("Todo was not found"),
            },
        };

        Ok(res.into())
    }

    async fn delete_todo(
        &self,
        req: Request<DeleteTodoRequest>,
    ) -> Result<Response<DeleteTodoResponse>, Status> {
        // TODO: Refactor me

        let DeleteTodoRequest { id } = req.into_inner();

        let Ok(id) = uuid::Uuid::try_from(id) else {
            return Err(Status::invalid_argument("Invalid todo ID"));
        };

        sqlx::query!(
            r#"
                DELETE FROM todos
                WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|error| {
            tracing::error!("Failed to delete todo from database: {error:?}");
            Status::internal("Failed to delete todo")
        })
        .map(|_| {
            DeleteTodoResponse {
                msg: String::from("Success"),
            }
            .into()
        })
    }

    async fn fetch_one_todo(
        &self,
        req: Request<FetchOneTodoRequest>,
    ) -> Result<Response<FetchOneTodoResponse>, Status> {
        // TODO: Refactor me

        let FetchOneTodoRequest { id } = req.into_inner();

        let Ok(id) = uuid::Uuid::try_from(id) else {
            return Err(Status::invalid_argument("Invalid todo ID"));
        };

        sqlx::query!(
            r#"
                SELECT title, description, is_done, updated_at
                FROM todos
                WHERE id = $1
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            tracing::error!("Database error: {error:?}");
            Status::internal("Internal server error")
        })
        .map(|maybe_row| {
            let Some(row) = maybe_row else {
                return FetchOneTodoResponse {
                    todo: None,
                    msg: String::from("Not found"),
                }
                .into();
            };

            let todo = TodoItem {
                id: id.to_string(),
                title: row.title,
                description: row.description,
                is_done: row.is_done,
                updated_at: Some(utils::chrono_offset_date_time_to_prost_timestamp(
                    row.updated_at,
                )),
            };

            FetchOneTodoResponse {
                todo: Some(todo),
                msg: String::new(),
            }
            .into()
        })
    }

    async fn fetch_many_todos(
        &self,
        req: Request<FetchManyTodosRequest>,
    ) -> Result<Response<FetchManyTodosResponse>, Status> {
        // TODO: Refactor me

        const TODOS_FETCH_LIMIT_MIN: u32 = 8;
        const TODOS_FETCH_LIMIT_MAX: u32 = 64;

        let FetchManyTodosRequest { offset, limit } = req.into_inner();

        let limit = limit.clamp(TODOS_FETCH_LIMIT_MIN, TODOS_FETCH_LIMIT_MAX);

        sqlx::query!(
            r#"
                SELECT id, title, description, is_done, updated_at
                FROM todos
                LIMIT $1
                OFFSET $2
            "#,
            limit as i32,
            offset as i32
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            tracing::error!("Failed to fetch todos. Database error: {error:?}");
            Status::internal("Failed to fetch todos")
        })
        .map(|rows| {
            let todos = rows
                .into_iter()
                .map(|todo| TodoItem {
                    id: todo.id.to_string(),
                    title: todo.title,
                    description: todo.description,
                    is_done: todo.is_done,
                    updated_at: Some(utils::chrono_offset_date_time_to_prost_timestamp(
                        todo.updated_at,
                    )),
                })
                .collect::<Vec<TodoItem>>();

            let count = todos.len() as u32;

            FetchManyTodosResponse { todos, count }.into()
        })
    }
}

#[cfg(test)]
mod todo_tests {
    use super::*;
    use crate::config::Config;

    #[inline]
    async fn init_todo_service() -> TodoService {
        let Config { db_url, .. } = Config::parse().await.unwrap();

        TodoService::init(&db_url)
            .await
            .expect("Failed to initialize `TodoService`.")
    }

    #[tokio::test]
    async fn create_todo() {
        let service = init_todo_service().await;

        let response = service.create_todo(Request::new(CreateTodoRequest {
            title: "Create todo".to_owned(),
            description: None,
        }));

        assert!(response.await.is_ok());
    }

    #[tokio::test]
    async fn create_and_update_todo() {
        let service = init_todo_service().await;

        // Create todo
        let todo = service
            .create_todo(Request::new(CreateTodoRequest {
                title: "Create todo".to_owned(),
                description: None,
            }))
            .await
            .expect("Create todo")
            .into_inner()
            .todo
            .unwrap();

        // Update todo
        let _update_response = service
            .update_todo(Request::new(UpdateTodoRequest {
                id: todo.id,
                title: Some("Update todo".to_owned()),
                description: None,
                is_done: Some(true),
            }))
            .await
            .expect("Expect: update todo");
    }

    #[tokio::test]
    async fn create_and_delete_todo() {
        let service = init_todo_service().await;

        // Create todo
        let todo = service
            .create_todo(Request::new(CreateTodoRequest {
                title: "Create todo".to_owned(),
                description: None,
            }))
            .await
            .expect("Create todo")
            .into_inner()
            .todo
            .unwrap();

        // Delete todo
        let _delete_response = service
            .delete_todo(Request::new(DeleteTodoRequest { id: todo.id }))
            .await
            .expect("Expect: delete todo");
    }

    #[tokio::test]
    async fn create_and_fetch_one_todo() {
        let service = init_todo_service().await;

        // Create todo
        let todo = service
            .create_todo(Request::new(CreateTodoRequest {
                title: "Create todo".to_owned(),
                description: None,
            }))
            .await
            .expect("Create todo")
            .into_inner()
            .todo
            .unwrap();

        // Fetch one todo
        let fetch_one_response = service
            .fetch_one_todo(Request::new(FetchOneTodoRequest { id: todo.id }))
            .await
            .expect("Expect: fetch one todo")
            .into_inner();

        assert!(fetch_one_response.todo.is_some());
    }

    #[tokio::test]
    async fn create_and_fetch_many_todos() {
        let service = init_todo_service().await;

        // Create todos
        let mut requests = vec![];

        const OFFSET: usize = 0;
        const LIMIT: usize = 10;

        for i in OFFSET..LIMIT {
            let req = service.create_todo(Request::new(CreateTodoRequest {
                title: format!("Todo #{i}"),
                description: None,
            }));

            requests.push(req);
        }

        for req in requests {
            let _ = req.await;
        }

        // Fetch many todos
        let FetchManyTodosResponse { todos, count } = service
            .fetch_many_todos(Request::new(FetchManyTodosRequest {
                offset: OFFSET as u32,
                limit: LIMIT as u32,
            }))
            .await
            .expect("Expect: fetch many todos")
            .into_inner();

        assert_eq!(todos.len(), count as usize);
    }
}

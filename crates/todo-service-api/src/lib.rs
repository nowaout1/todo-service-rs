use todo_service_domain::error::ValidationTodoError;
use tonic::{Request, Response, Status};

use todo_service_domain::entity;
use todo_service_domain::repository;
use todo_service_domain::use_cases;

pub mod todo_proto {
    tonic::include_proto!("todo.v1");
}

type Result<T> = std::result::Result<Response<T>, Status>;

impl Into<todo_proto::TodoItem> for entity::TodoEntity {
    fn into(self) -> todo_proto::TodoItem {
        todo_proto::TodoItem {
            id: self.id.to_string(),
            title: self.title,
            description: self.description,
            is_done: self.is_done,
            updated_at: Some(prost_types::Timestamp {
                seconds: self.updated_at.unix_timestamp(),
                nanos: self.updated_at.nanosecond() as i32,
            }),
        }
    }
}

pub struct TodoServiceApi<Repository> {
    repository: Repository,
}

impl<Repository> TodoServiceApi<Repository> {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

#[tonic::async_trait]
impl<Repository> todo_proto::todo_server::Todo for TodoServiceApi<Repository>
where
    Repository: repository::FetchTodoById
        + repository::FetchTodosByRange
        + repository::CreateTodo
        + repository::UpdateTodo
        + repository::DeleteTodo
        + Send
        + Sync
        + 'static,
{
    async fn fetch_todo_by_id(
        &self,
        req: Request<todo_proto::FetchTodoByIdRequest>,
    ) -> Result<todo_proto::FetchTodoByIdResponse> {
        let todo_proto::FetchTodoByIdRequest { id } = req.into_inner();

        let id = utils::try_parse_id(&id)?;

        match use_cases::fetch_todo_by_id(&self.repository, id).await {
            // Return todo if exists
            Ok(Some(todo)) => Ok(todo_proto::FetchTodoByIdResponse {
                todo: Some(todo.into()),
                msg: String::new(),
            }),
            // Return none if not exists
            Ok(None) => Ok(todo_proto::FetchTodoByIdResponse {
                todo: None,
                msg: String::from("not found"),
            }),
            Err(error) => Err(status::internal_server_error(
                "failed to fetch todo by id",
                error,
            )),
        }
        .map(Into::into)
    }

    async fn fetch_todos_by_range(
        &self,
        req: Request<todo_proto::FetchTodosByRangeRequest>,
    ) -> Result<todo_proto::FetchTodosByRangeResponse> {
        let todo_proto::FetchTodosByRangeRequest { offset, limit } = req.into_inner();

        let todos = use_cases::fetch_todos_by_range(&self.repository, offset as _, limit as _)
            .await
            .map_err(|error| {
                status::internal_server_error("failed to fetch todos by range", error)
            })?;

        Ok(todo_proto::FetchTodosByRangeResponse {
            count: todos.len() as _,
            todos: todos.into_iter().map(Into::into).collect(),
        })
        .map(Into::into)
    }

    async fn create_todo(
        &self,
        req: Request<todo_proto::CreateTodoRequest>,
    ) -> Result<todo_proto::CreateTodoResponse> {
        let todo_proto::CreateTodoRequest { title, description } = req.into_inner();

        let result =
            use_cases::create_todo(&self.repository, title, description.unwrap_or_default()).await;

        // Handle internal error
        if let Err(ValidationTodoError::Internal(internal_error)) = result {
            return Err(status::internal_server_error(
                "failed to create todo",
                internal_error,
            ));
        }

        result
            .map(|todo| todo_proto::CreateTodoResponse {
                todo: Some(todo.into()),
                msg: "created".to_owned(),
            })
            .map(Into::into)
            .map_err(Status::invalid_argument)
    }

    async fn update_todo(
        &self,
        req: Request<todo_proto::UpdateTodoRequest>,
    ) -> Result<todo_proto::UpdateTodoResponse> {
        let todo_proto::UpdateTodoRequest {
            id,
            title,
            description,
            is_done,
        } = req.into_inner();

        let id = utils::try_parse_id(&id)?;

        let result =
            use_cases::update_todo(&self.repository, id, title, description, is_done).await;

        // Handle internal error
        if let Err(ValidationTodoError::Internal(internal_error)) = result {
            return Err(status::internal_server_error(
                "failed to update todo",
                internal_error,
            ));
        }

        result
            .map(|todo| todo_proto::UpdateTodoResponse {
                todo: Some(todo.into()),
                msg: "updated".to_owned(),
            })
            .map(Into::into)
            .map_err(Status::invalid_argument)
    }

    async fn delete_todo(
        &self,
        req: Request<todo_proto::DeleteTodoRequest>,
    ) -> Result<todo_proto::DeleteTodoResponse> {
        let todo_proto::DeleteTodoRequest { id } = req.into_inner();

        let id = utils::try_parse_id(&id)?;

        use_cases::delete_todo(&self.repository, id)
            .await
            .map_err(|error| status::internal_server_error("failed to delete todo", error))?;

        Ok(todo_proto::DeleteTodoResponse {
            msg: "success".to_owned(),
        })
        .map(Into::into)
    }
}

mod utils {
    pub(crate) fn try_parse_id(id: &str) -> Result<uuid::Uuid, tonic::Status> {
        uuid::Uuid::try_parse(id.into())
            .map_err(|_| tonic::Status::invalid_argument("invalid todo ID"))
    }
}

mod status {
    pub(crate) fn internal_server_error<Error>(cause: &str, error: Error) -> tonic::Status
    where
        Error: std::fmt::Debug,
    {
        tracing::error!("{cause}: {error:?}");
        tonic::Status::internal("internal server error")
    }
}

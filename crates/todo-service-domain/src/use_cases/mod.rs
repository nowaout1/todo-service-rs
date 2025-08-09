use crate::entity;
use crate::error;
use crate::repository;
use crate::validator;

pub async fn fetch_todo_by_id<Repository, RepositoryError>(
    repository: &Repository,
    id: uuid::Uuid,
) -> Result<Option<entity::TodoEntity>, RepositoryError>
where
    Repository: repository::FetchTodoById<Error = RepositoryError>,
    RepositoryError: std::fmt::Debug,
{
    Ok(repository
        .fetch_todo_by_id(repository::FetchTodoByIdRequest { id })
        .await?
        .map(|res| res.into()))
}

pub async fn fetch_todos_by_range<Repository, RepositoryError>(
    repository: &Repository,
    offset: usize,
    limit: usize,
) -> Result<Vec<entity::TodoEntity>, RepositoryError>
where
    Repository: repository::FetchTodosByRange<Error = RepositoryError>,
    RepositoryError: std::fmt::Debug,
{
    Ok(repository
        .fetch_todos_by_range(repository::FetchTodosByRangeRequest { offset, limit })
        .await?
        .into_iter()
        .map(|items| items.into())
        .collect())
}

pub async fn create_todo<Repository, RepositoryError>(
    repository: &Repository,
    title: String,
    description: String,
) -> Result<entity::TodoEntity, error::ValidationTodoError<RepositoryError>>
where
    Repository: repository::CreateTodo<Error = RepositoryError>,
    RepositoryError: std::fmt::Debug,
{
    validator::validate_title(&title)?;
    validator::validate_description(&description)?;

    Ok(repository
        .create_todo(repository::CreateTodoRequest { title, description })
        .await?
        .into())
}

pub async fn update_todo<Repository, RepositoryError>(
    repository: &Repository,
    id: uuid::Uuid,
    title: Option<String>,
    description: Option<String>,
    is_done: Option<bool>,
) -> Result<entity::TodoEntity, error::ValidationTodoError<RepositoryError>>
where
    Repository: repository::UpdateTodo<Error = RepositoryError>,
    RepositoryError: std::fmt::Debug,
{
    if title.is_none() && description.is_none() && is_done.is_none() {
        return Err(error::ValidationTodoError::NoChanges);
    }

    if let Some(title) = &title {
        validator::validate_title(title)?;
    }

    if let Some(description) = &description {
        validator::validate_description(description)?;
    }

    Ok(repository
        .update_todo(repository::UpdateTodoRequest {
            id,
            title,
            description,
            is_done,
        })
        .await?
        .into())
}

pub async fn delete_todo<Repository, RepositoryError>(
    repository: &Repository,
    id: uuid::Uuid,
) -> Result<(), RepositoryError>
where
    Repository: repository::DeleteTodo<Error = RepositoryError>,
    RepositoryError: std::fmt::Debug,
{
    repository
        .delete_todo(repository::DeleteTodoRequest { id })
        .await?;

    Ok(())
}

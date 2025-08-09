use crate::constants;
use crate::error;

#[inline]
pub fn validate_title<InternalError>(
    title: impl Into<String>,
) -> std::result::Result<(), error::ValidationTodoError<InternalError>>
where
    InternalError: std::fmt::Debug,
{
    match title.into().len() {
        x if x < constants::MIN_TITLE_LENGTH => Err(error::ValidationTodoError::TitleIsTooShort),
        x if x > constants::MAX_TITLE_LENGTH => Err(error::ValidationTodoError::TitleIsTooLong),
        _ => Ok(()),
    }
}

#[inline]
pub fn validate_description<InternalError>(
    description: impl Into<String>,
) -> std::result::Result<(), error::ValidationTodoError<InternalError>>
where
    InternalError: std::fmt::Debug,
{
    match description.into().len() {
        x if x > constants::MAX_DESCRIPTION_LENGTH => {
            Err(error::ValidationTodoError::DescriptionIsTooLong)
        }
        _ => Ok(()),
    }
}

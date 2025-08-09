use crate::constants;

#[derive(thiserror::Error, Debug)]
pub enum ValidationTodoError<InternalError> {
    #[error("min length of title is {}", constants::MIN_TITLE_LENGTH)]
    TitleIsTooShort,

    #[error("max length of title is {}", constants::MAX_TITLE_LENGTH)]
    TitleIsTooLong,

    #[error("max length of description is {}", constants::MAX_DESCRIPTION_LENGTH)]
    DescriptionIsTooLong,

    #[error("no data for changes provided")]
    NoChanges,

    #[error("internal server error")]
    Internal(#[from] InternalError),
}

impl<InternalError> From<ValidationTodoError<InternalError>> for String {
    fn from(value: ValidationTodoError<InternalError>) -> Self {
        value.to_string()
    }
}

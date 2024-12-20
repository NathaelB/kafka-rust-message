use std::sync::Arc;
use axum::Extension;
use crate::application::http::handlers::{ApiError, ApiSuccess};
use axum::extract::Path;
use axum::http::StatusCode;
use serde::Serialize;
use tracing::info;
use crate::domain::message::models::{Message, MessageError};
use crate::domain::message::ports::MessageService;

#[derive(Debug, Clone, Serialize)]
pub struct GetMessagesResponseData(Vec<Message>);

impl From<Vec<Message>> for GetMessagesResponseData {
    fn from(value: Vec<Message>) -> Self {
        GetMessagesResponseData(value)
    }
}

impl From<MessageError> for ApiError {
    fn from(value: MessageError) -> Self {
        match value {
            MessageError::UnknownError => Self::InternalServerError("unknown error".to_string()),
        }
    }
}

pub async fn list_servers<M>(
    Path(server_id): Path<String>,
    Extension(message_service): Extension<Arc<M>>
) -> Result<ApiSuccess<Vec<Message>>, ApiError>
where
    M: MessageService
{

    info!("Listing messages for server: {}", server_id);

    message_service
        .find_by_server_id(server_id)
        .await
        .map_err(ApiError::from)
        .map(|messages| ApiSuccess::new(StatusCode::OK, messages.into()))
}

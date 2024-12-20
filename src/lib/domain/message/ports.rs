use crate::domain::message::models::{Message, MessageError};
use std::future::Future;

pub trait MessageService: Clone + Send + Sync + 'static {
    fn find_by_server_id(
        &self,
        server_id: String,
    ) -> impl Future<Output = Result<Vec<Message>, MessageError>> + Send;
}

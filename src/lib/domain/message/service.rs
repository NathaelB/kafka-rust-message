use crate::application::ports::messaging_ports::{MessagingPort, MessagingTypeImpl};
use crate::domain::message::models::{Message, MessageError};
use crate::domain::message::ports::MessageService;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MessageServiceImpl {
    pub messaging: Arc<MessagingTypeImpl>,
}

impl MessageServiceImpl {
    pub fn new(messaging: Arc<MessagingTypeImpl>) -> Self {
        Self { messaging }
    }
}

impl MessageService for MessageServiceImpl {
    async fn find_by_server_id(&self, server_id: String) -> Result<Vec<Message>, MessageError> {
        let topic = format!("server-{}-messages", server_id);

        let messages = self.messaging.get_messages(topic).await
            .map_err(|_| MessageError::UnknownError)?;

        let messages: Vec<Message> = messages
            .iter()
            .map(|message| serde_json::from_str(message).unwrap())
            .collect();

        Ok(messages)
    }
}

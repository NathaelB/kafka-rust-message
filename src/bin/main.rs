use std::sync::Arc;
use log::info;
use poc::application::http::{HttpServer, HttpServerConfig};
use poc::application::ports::messaging_ports::{MessagingPort, MessagingType, MessagingTypeImpl};
use serde::{Deserialize, Serialize};
use poc::domain::message::service::MessageServiceImpl;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMessageEvent {
    content: String,
    user_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let topics = "test";

    let kafka = MessagingTypeImpl::new(&MessagingType::Kafka).await?;
    let kafka = Arc::new(kafka);

    kafka
        .subscribe(topics, |message: NewMessageEvent| async move {
            info!("Message received: {:?}", message);
            Ok(())
        })
        .await?;

    let message_service = Arc::new(MessageServiceImpl::new(Arc::clone(&kafka)));

    let server_config = HttpServerConfig::new("3333".to_string());

    let http_server = HttpServer::new(server_config, Arc::clone(&message_service)).await?;

    http_server.run().await?;

    Ok(())
}

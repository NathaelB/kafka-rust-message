use log::info;
use serde::{Deserialize, Serialize};
use poc::application::http::{HttpServer, HttpServerConfig};
use poc::application::ports::messaging_ports::{MessagingPort, MessagingType, MessagingTypeImpl};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMessageEvent {
    content: String,
    user_id: String
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    tracing_subscriber::fmt::init();
    let topics = "test";

    let kafka = MessagingTypeImpl::new(&MessagingType::Kafka).await?;

    kafka.subscribe(topics, |message: NewMessageEvent| async move {
        info!("Message received: {:?}", message);
        Ok(())
    }).await?;

    let server_config = HttpServerConfig::new("3333".to_string());

    let http_server = HttpServer::new(server_config).await?;

    http_server.run().await?;

    Ok(())
}
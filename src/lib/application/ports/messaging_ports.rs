use crate::infrastructure::messaging::kafka::KafkaMessaging;
use anyhow::Result;
use clap::builder::PossibleValue;
use clap::ValueEnum;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::future::Future;

#[derive(Debug, Clone)]
pub enum MessagingType {
    Kafka,
}

impl ValueEnum for MessagingType {
    fn value_variants<'a>() -> &'a [Self] {
        &[MessagingType::Kafka]
    }

    fn from_str(input: &str, _ignore_case: bool) -> std::result::Result<Self, String> {
        match input {
            "kafka" => Ok(MessagingType::Kafka),
            _ => Err("Invalid messaging type".to_string()),
        }
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            MessagingType::Kafka => Some(PossibleValue::new("kafka")),
        }
    }
}

#[derive(Clone)]
pub enum MessagingTypeImpl {
    Kafka(KafkaMessaging),
}

impl Debug for MessagingTypeImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessagingTypeImpl::Kafka(_) => write!(f, "Kafka"),
        }
    }
}

impl MessagingTypeImpl {
    pub async fn new(typ: &MessagingType) -> Result<Self> {
        match typ {
            MessagingType::Kafka => {
                let broker = "localhost:9092".to_string();
                let group_id = "my-group".to_string();
                let messaging = KafkaMessaging::new(broker, group_id)?;

                Ok(MessagingTypeImpl::Kafka(messaging))
            }
        }
    }
}

impl MessagingPort for MessagingTypeImpl {
    async fn publish_message(&self, topic: String, message: String) -> Result<()> {
        match self {
            MessagingTypeImpl::Kafka(messaging) => messaging.publish_message(topic, message).await,
        }
    }

    async fn subscribe<F, T, Fut>(&self, topic: &str, handler: F) -> Result<()>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        T: DeserializeOwned + Send + Sync + Debug + Clone + 'static,
    {
        match self {
            MessagingTypeImpl::Kafka(messaging) => messaging.subscribe(topic, handler).await,
        }
    }

    async fn get_messages(&self, topic: String) -> Result<Vec<String>> {
        match self {
            MessagingTypeImpl::Kafka(messaging) => messaging.get_messages(topic).await,
        }
    }
}

pub trait MessagingPort: Clone + Send + Sync + 'static {
    fn publish_message(
        &self,
        topic: String,
        message: String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn subscribe<F, T, Fut>(
        &self,
        topic: &str,
        handler: F,
    ) -> impl Future<Output = anyhow::Result<()>> + Send
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
        T: DeserializeOwned + Send + Sync + Debug + Clone + 'static;

    fn get_messages(
        &self,
        topic: String,
    ) -> impl Future<Output = anyhow::Result<Vec<String>>> + Send;
}

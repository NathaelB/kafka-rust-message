use crate::application::ports::messaging_ports::MessagingPort;
use anyhow::Result;
use futures::StreamExt;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::{ClientConfig, Message};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;

#[derive(Clone)]
pub struct KafkaMessaging {
    producer: Arc<FutureProducer>,
    consumer: Arc<StreamConsumer>,
    brokers: String,
    group_id: String,
}

impl KafkaMessaging {
    pub fn new(brokers: String, group_id: String) -> Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .create::<FutureProducer>()?;

        let consumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("group.id", &group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create::<StreamConsumer>()?;

        Ok(KafkaMessaging {
            producer: Arc::new(producer),
            consumer: Arc::new(consumer),
            brokers,
            group_id,
        })
    }
}

impl MessagingPort for KafkaMessaging {
    async fn publish_message(&self, topic: String, message: String) -> Result<()> {
        let record = FutureRecord::to(&topic).payload(&message).key("order");

        match self
            .producer
            .send(record, std::time::Duration::from_secs(0))
            .await
        {
            Ok(delivery) => {
                println!("Message delivered to {:?}", delivery);
                Ok(())
            }
            Err((e, _)) => {
                println!("Failed to deliver message: {:?}", e);
                Err(anyhow::anyhow!("Failed to deliver message"))
            }
        }
    }

    async fn subscribe<F, T, Fut>(&self, topic: &str, handler: F) -> Result<()>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        T: DeserializeOwned + Send + Sync + Debug + Clone + 'static,
    {
        self.consumer.subscribe(&[topic])?;

        let consumer = Arc::clone(&self.consumer);

        tokio::spawn(async move {
            while let Some(result) = consumer.stream().next().await {
                match result {
                    Ok(message) => {
                        if let Some(payload) = message.payload_view::<str>() {
                            match payload {
                                Ok(text) => {
                                    let parsed_message: T = match serde_json::from_str(text) {
                                        Ok(msg) => msg,
                                        Err(e) => {
                                            tracing::error!("Failed to parse message: {:?}", e);
                                            continue;
                                        }
                                    };

                                    if let Err(e) = handler(parsed_message).await {
                                        tracing::error!("Failed to handle message: {:?}", e);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to decode message payload: {:?}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Kafka error: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn get_messages(&self, topic: String) -> Result<Vec<String>> {
        self.consumer.subscribe(&[&topic])?;
        let mut stream = self.consumer.stream();
        let mut messages = Vec::new();

        println!("Getting messages from topic: {:?}", topic);

        while let Some(result) = stream.next().await {
            println!("Received message: {:?}", result);
            match result {
                Ok(message) => {
                    if let Some(payload) = message.payload_view::<str>() {
                        if let Ok(text) = payload {
                            println!("Received message: {:?}", text);
                            messages.push(text.to_string());
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Kafka error: {:?}", e);
                }
            }
        }
        Ok(messages)
    }
}

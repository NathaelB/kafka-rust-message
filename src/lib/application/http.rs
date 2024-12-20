mod handlers;
mod responses;

use std::sync::Arc;
use crate::application::http::handlers::list_messages::list_servers;
use anyhow::Context;
use axum::Extension;
use axum::routing::get;
use log::info;
use tokio::net;
use tracing::info_span;
use crate::domain::message::ports::MessageService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerConfig {
    pub port: String,
}

impl HttpServerConfig {
    pub fn new(port: String) -> Self {
        Self { port }
    }
}

#[derive(Debug, Clone)]
struct AppState<M>
where
    M: MessageService,
{
    message_service: Arc<M>,
}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    pub async fn new<M>(config: HttpServerConfig, message_service: Arc<M>) -> anyhow::Result<Self>
    where
        M: MessageService,
    {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request| {
                let uri: String = request.uri().to_string();
                info_span!("http_request", method = ?request.method(), uri)
            },
        );

        let state = AppState {
            message_service,
        };


        let router = axum::Router::new()
            .nest("", api_routes())
            .layer(trace_layer)
            .layer(Extension(Arc::clone(&state.message_service)))
            .with_state(state);

        let listener = net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
            .await
            .with_context(|| format!("Failed to bind to port {}", config.port))?;

        Ok(Self { router, listener })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        info!(
            "Server is running on http://{}",
            self.listener.local_addr()?
        );
        axum::serve(self.listener, self.router)
            .await
            .context("received error while running server")?;

        Ok(())
    }
}

fn api_routes<M>() -> axum::Router<AppState<M>>
where
    M: MessageService,
{
    axum::Router::new().route("/servers/:id/messages", get(list_servers::<M>))
}

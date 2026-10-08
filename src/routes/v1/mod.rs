use crate::{
    alertmanager::{self, Payload},
    ntfy::Message,
    ServerState,
};
use axum::{extract::State, http::StatusCode, routing::post, Router};

mod error;

use error::Error;

pub fn router() -> Router<ServerState> {
    Router::new().nest(
        "/v1",
        Router::new().route("/webhooks/alerts", post(webhook_alerts)),
    )
}

async fn webhook_alerts(
    State(state): State<ServerState>,
    body: String,
) -> Result<StatusCode, Error> {
    log::debug!("received request body: {body}");

    let payload: Payload = serde_json::from_str(&body)?;

    let context = tera::Context::from_serialize(&payload)?;
    let topic = match &state.config.topic.label {
        Some(key) => match payload.get_common_label(key) {
            Some(value) => value,
            None => &state.config.topic.default,
        },
        None => &state.config.topic.default,
    };

    let mut msg = Message::new(topic)
        .title(&state.tera.render("title", &context)?)
        .message(&state.tera.render("message", &context)?)
        .markdown(true);

    match payload.status {
        alertmanager::Status::Firing => {
            msg = msg.tag("warning");
        }
        alertmanager::Status::Resolved => {
            msg = msg.tag("tada");
        }
    }

    state.ntfy.send(&msg).await?;

    Ok(StatusCode::OK)
}

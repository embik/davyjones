use crate::{
    alertmanager::{self, Payload},
    ntfy::Message,
    ServerState,
};
use actix_web::{web, HttpResponse, Responder};

mod error;

use error::Error;

pub fn scope() -> actix_web::Scope {
    web::scope("/v1").route("/webhooks/alerts", web::post().to(webhook_alerts))
}

async fn webhook_alerts(
    payload: web::Json<Payload>,
    state: web::Data<ServerState>,
) -> Result<impl Responder, Error> {
    log::debug!("received request");

    let context = tera::Context::from_serialize(&payload)?;
    let topic = match &state.config.topic.label {
        Some(key) => match payload.get_common_label(&key) {
            Some(value) => value,
            None => &state.config.topic.default,
        },
        None => &state.config.topic.default,
    };

    let mut msg = Message::new(&topic)
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

    Ok(HttpResponse::Ok().finish())
}

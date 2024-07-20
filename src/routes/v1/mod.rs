use crate::{alertmanager::Payload, ntfy::Message, ServerState};
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

    let msg = Message::new(&topic)
        .title(&state.tera.render("title", &context)?)
        .message(&state.tera.render("message", &context)?)
        .markdown(true);

    Ok(HttpResponse::Ok().body(format!("{}", msg.title.unwrap_or("lol".to_string()))))
}

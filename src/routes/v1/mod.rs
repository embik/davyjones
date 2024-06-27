use crate::alertmanager::Payload;
use crate::config::Config;
use actix_web::{web, Responder};

pub fn scope() -> actix_web::Scope {
    web::scope("/v1").route("/webhooks/alerts", web::post().to(webhook_alerts))
}

async fn webhook_alerts(payload: web::Json<Payload>, config: web::Data<Config>) -> impl Responder {
    format!("Hello world: {}: {}", config.nfty.server, payload.version)
}

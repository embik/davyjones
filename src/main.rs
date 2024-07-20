use ntfy::Message;
use std::path::Path;
use tera::Tera;

use actix_web::{web::Data, App, HttpServer};
use anyhow::Result;

pub mod alertmanager;
pub mod ntfy;

mod cmd;
mod config;
mod routes;

#[derive(Debug, Clone)]
struct ServerState {
    config: config::Config,
    tera: Tera,
}

#[actix_web::main]
async fn main() -> Result<()> {
    let matches = cmd::cli().get_matches();
    setup_logger(matches.get_flag("verbose"))?;

    let config_path = match matches.get_one::<std::path::PathBuf>("config") {
        Some(path) => path.clone(),
        None => Path::new(config::DEFAULT_CONFIG_FILE).to_path_buf(),
    };
    log::debug!("using {} as configuration file", config_path.display());
    let conf = config::load(&config_path)?;
    log::info!("loaded config from {}", config_path.display());

    let title_template = match &conf.templates {
        Some(tmpls) => match &tmpls.title {
            Some(title_template) => &title_template,
            None => config::DEFAULT_TITLE_TEMPLATE,
        },
        None => config::DEFAULT_TITLE_TEMPLATE,
    };
    let message_template = match &conf.templates {
        Some(tmpls) => match &tmpls.message {
            Some(message_template) => &message_template,
            None => config::DEFAULT_MESSAGE_TEMPLATE,
        },
        None => config::DEFAULT_MESSAGE_TEMPLATE,
    };

    // set up tera templating engine
    let mut tera = Tera::default();
    tera.add_raw_template("title", &title_template)?;
    tera.add_raw_template("message", &message_template)?;

    let state = ServerState { config: conf, tera };

    // start actix http server
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .service(routes::v1::scope())
    })
    .bind((
        matches
            .get_one::<String>("host")
            .unwrap_or(&("localhost".to_string()))
            .clone(),
        matches.get_one::<u16>("port").unwrap_or(&8080).clone(),
    ))?
    .run()
    .await?;

    Ok(())
}

fn setup_logger(verbose: bool) -> Result<()> {
    let filter_level = match verbose {
        true => log::LevelFilter::Debug,
        false => log::LevelFilter::Info,
    };

    Ok(env_logger::Builder::new()
        .filter_level(filter_level)
        .format_target(false)
        .try_init()?)
}

use anyhow::Result;
use axum::{extract::Request, middleware, middleware::Next, response::Response, Router};
use tera::Tera;

use std::path::Path;

pub mod alertmanager;
pub mod ntfy;

mod cmd;
mod config;
mod routes;

#[derive(Clone)]
struct ServerState {
    config: config::Config,
    ntfy: ntfy::Ntfy,
    tera: Tera,
}

#[tokio::main]
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
            Some(title_template) => title_template,
            None => config::DEFAULT_TITLE_TEMPLATE,
        },
        None => config::DEFAULT_TITLE_TEMPLATE,
    };
    let message_template = match &conf.templates {
        Some(tmpls) => match &tmpls.message {
            Some(message_template) => message_template,
            None => config::DEFAULT_MESSAGE_TEMPLATE,
        },
        None => config::DEFAULT_MESSAGE_TEMPLATE,
    };

    // set up tera templating engine
    let mut tera = Tera::default();
    tera.add_raw_template("title", title_template)?;
    tera.add_raw_template("message", message_template)?;

    let auth = if conf.ntfy.username.is_some() && conf.ntfy.password.is_some() {
        Some((
            conf.ntfy.username.clone().unwrap_or_default(),
            conf.ntfy.password.clone().unwrap_or_default(),
        ))
    } else {
        None
    };
    let ntfy = ntfy::Ntfy::new(
        &conf.ntfy.url,
        auth.as_ref().map(|(u, p)| (u.as_str(), p.as_str())),
    );

    let state = ServerState {
        config: conf,
        tera,
        ntfy,
    };

    let app = Router::new()
        .merge(routes::v1::router())
        .layer(middleware::from_fn(log_request))
        .with_state(state);

    let host = matches
        .get_one::<String>("host")
        .map(|h| h.as_str())
        .unwrap_or("localhost");
    let port = matches.get_one::<u16>("port").copied().unwrap_or(8080);

    // bind all addresses the host resolves to, like actix did
    let addrs = tokio::net::lookup_host(format!("{host}:{port}")).await?;
    let mut listeners = Vec::new();
    let mut last_err = None;
    for addr in addrs {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                log::info!("listening on {addr}");
                listeners.push(listener);
            }
            Err(err) => last_err = Some(err),
        }
    }
    if listeners.is_empty() {
        return Err(match last_err {
            Some(err) => err.into(),
            None => std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                "host resolved to no addresses",
            )
            .into(),
        });
    }

    let mut servers = Vec::new();
    for listener in listeners {
        let app = app.clone();
        servers.push(tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await
        }));
    }
    for server in servers {
        server.await??;
    }

    Ok(())
}

async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    log::info!(
        "{} {} -> {} ({}ms)",
        method,
        path,
        response.status(),
        start.elapsed().as_millis()
    );
    response
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let sigterm = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut sigterm) => sigterm.recv().await,
            Err(err) => {
                log::warn!("could not install SIGTERM handler: {err}");
                std::future::pending().await
            }
        }
    };
    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm => {},
    }
    log::info!("received shutdown signal, stopping server");
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

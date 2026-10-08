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

    let app = app(state);

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

fn app(state: ServerState) -> Router {
    Router::new()
        .merge(routes::v1::router())
        .layer(middleware::from_fn(log_request))
        .with_state(state)
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{header, HeaderMap, StatusCode},
        routing::post,
    };
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;

    type Received = Arc<Mutex<Vec<(String, serde_json::Value)>>>;

    fn test_state(ntfy_url: &str) -> ServerState {
        let conf = config::Config {
            ntfy: config::Ntfy {
                url: ntfy_url.to_string(),
                username: Some("user".to_string()),
                password: Some("secret".to_string()),
            },
            topic: config::Topic {
                default: "prometheusalerts".to_string(),
                label: Some("topic".to_string()),
            },
            templates: None,
        };

        let mut tera = Tera::default();
        tera.add_raw_template("title", config::DEFAULT_TITLE_TEMPLATE)
            .unwrap();
        tera.add_raw_template("message", config::DEFAULT_MESSAGE_TEMPLATE)
            .unwrap();

        ServerState {
            ntfy: ntfy::Ntfy::new(ntfy_url, Some(("user", "secret"))),
            tera,
            config: conf,
        }
    }

    // mock ntfy server recording the Authorization header and JSON body
    async fn mock_ntfy() -> (String, Received) {
        let received: Received = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&received);

        let mock: Router = Router::new().route(
            "/",
            post(move |headers: HeaderMap, body: String| {
                let captured = Arc::clone(&captured);
                async move {
                    let auth = headers
                        .get(header::AUTHORIZATION)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or_default()
                        .to_string();
                    let message: serde_json::Value = serde_json::from_str(&body).unwrap();
                    captured.lock().unwrap().push((auth, message));
                    StatusCode::OK
                }
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, mock).await.unwrap();
        });

        (format!("http://{addr}/"), received)
    }

    async fn start_app(state: ServerState) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app(state)).await.unwrap();
        });
        format!("http://{addr}")
    }

    // reserve a port and release it, so nothing listens on it
    async fn closed_url() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        format!("http://{addr}/")
    }

    #[tokio::test]
    async fn webhook_forwards_rendered_alert_to_ntfy() {
        let (ntfy_url, received) = mock_ntfy().await;
        let base = start_app(test_state(&ntfy_url)).await;

        let response = reqwest::Client::new()
            .post(format!("{base}/v1/webhooks/alerts"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(include_str!("alertmanager/test/payload.json"))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let received = received.lock().unwrap();
        assert_eq!(received.len(), 1);
        let (auth, message) = &received[0];
        assert_eq!(auth, "Basic dXNlcjpzZWNyZXQ=");
        assert_eq!(message["topic"], "prometheusalerts");
        assert_eq!(message["title"], "UNKNOWN: Test");
        assert_eq!(message["tags"], serde_json::json!(["warning"]));
        assert_eq!(message["markdown"], true);
    }

    #[tokio::test]
    async fn webhook_rejects_invalid_json() {
        let (ntfy_url, _) = mock_ntfy().await;
        let base = start_app(test_state(&ntfy_url)).await;

        let response = reqwest::Client::new()
            .post(format!("{base}/v1/webhooks/alerts"))
            .header(header::CONTENT_TYPE, "application/json")
            .body("{invalid")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 400);
    }

    #[tokio::test]
    async fn webhook_returns_500_when_ntfy_unreachable() {
        let base = start_app(test_state(&closed_url().await)).await;

        let response = reqwest::Client::new()
            .post(format!("{base}/v1/webhooks/alerts"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(include_str!("alertmanager/test/payload.json"))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 500);
    }

    #[tokio::test]
    async fn webhook_handles_alert_without_annotations() {
        let (ntfy_url, received) = mock_ntfy().await;
        let base = start_app(test_state(&ntfy_url)).await;

        let mut payload: serde_json::Value =
            serde_json::from_str(include_str!("alertmanager/test/payload.json")).unwrap();
        payload["alerts"][0]
            .as_object_mut()
            .unwrap()
            .remove("annotations");

        let response = reqwest::Client::new()
            .post(format!("{base}/v1/webhooks/alerts"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(payload.to_string())
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let received = received.lock().unwrap();
        assert_eq!(received.len(), 1);
        let message = received[0].1["message"].as_str().unwrap();
        assert!(message.contains("no description given"));
    }

    #[tokio::test]
    async fn webhook_handles_empty_common_labels() {
        let (ntfy_url, received) = mock_ntfy().await;
        let base = start_app(test_state(&ntfy_url)).await;

        let mut payload: serde_json::Value =
            serde_json::from_str(include_str!("alertmanager/test/payload.json")).unwrap();
        payload["commonLabels"] = serde_json::json!({});

        let response = reqwest::Client::new()
            .post(format!("{base}/v1/webhooks/alerts"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(payload.to_string())
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let received = received.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].1["title"], "UNKNOWN: unknown");
    }

    #[tokio::test]
    async fn webhook_rejects_wrong_shape_json() {
        let (ntfy_url, _) = mock_ntfy().await;
        let base = start_app(test_state(&ntfy_url)).await;

        let response = reqwest::Client::new()
            .post(format!("{base}/v1/webhooks/alerts"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(r#"{"foo": "bar"}"#)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 400);
    }
}

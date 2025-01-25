use awc::{http::header, Client, Connector};
use rustls::{ClientConfig, RootCertStore};

use std::sync::Arc;

mod error;
mod message;

pub use error::Error;
pub use message::Message;

#[derive(Clone)]
pub struct Ntfy {
    url: String,
    use_basic_auth: bool,
    username: String,
    password: String,
    client: awc::Client,
}

impl Ntfy {
    pub fn new(url: &str, basic_auth: Option<(&String, &String)>) -> Ntfy {
        let client_tls_config = Arc::new(rustls_config());

        let client = Client::builder()
            .add_default_header((header::USER_AGENT, "davyjones-webhook-server/0.0"))
            // a "connector" wraps the stream into an encrypted connection
            .connector(Connector::new().rustls_0_23(Arc::clone(&client_tls_config)))
            .finish();

        let (use_basic_auth, username, password) = match basic_auth {
            Some((username, password)) => (true, username.clone(), password.clone()),
            None => (false, "".to_string(), "".to_string()),
        };

        Ntfy {
            url: url.to_string(),
            use_basic_auth,
            username,
            password,
            client,
        }
    }

    pub async fn send(&self, msg: &Message) -> Result<(), Error> {
        let mut request = self.client.post(&self.url);

        if self.use_basic_auth {
            request = request.basic_auth(&self.username, &self.password);
        }

        let response = request.send_json(msg).await?;

        if !response.status().is_success() {
            return Err(Error::ServerResponse(response.status()));
        }

        Ok(())
    }
}

fn rustls_config() -> ClientConfig {
    let root_store = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.to_owned());

    rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

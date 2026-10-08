use reqwest::{header, Client};

mod error;
mod message;

pub use error::Error;
pub use message::Message;

#[derive(Clone)]
pub struct Ntfy {
    url: String,
    username: Option<String>,
    password: Option<String>,
    client: Client,
}

impl Ntfy {
    pub fn new(url: &str, basic_auth: Option<(&str, &str)>) -> Ntfy {
        let client = Client::builder()
            .default_headers({
                let mut headers = header::HeaderMap::new();
                headers.insert(
                    header::USER_AGENT,
                    header::HeaderValue::from_static("davyjones-webhook-server/0.0"),
                );
                headers
            })
            .build()
            .expect("failed to build HTTP client");

        let (username, password) = match basic_auth {
            Some((username, password)) => (Some(username.to_string()), Some(password.to_string())),
            None => (None, None),
        };

        Ntfy {
            url: url.to_string(),
            username,
            password,
            client,
        }
    }

    pub async fn send(&self, msg: &Message) -> Result<(), Error> {
        let mut request = self.client.post(&self.url);

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.json(msg).send().await?;

        if !response.status().is_success() {
            return Err(Error::ServerResponse(response.status()));
        }

        Ok(())
    }
}

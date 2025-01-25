use serde::Deserialize;
use std::path::Path;

pub use error::Error;

pub const DEFAULT_CONFIG_FILE: &str = "/etc/davyjones/config.toml";
pub const DEFAULT_TITLE_TEMPLATE: &str = "{% if status == 'resolved' %}[Resolved] {% endif %}{{ commonLabels | get(key='severity', default='unknown') | upper}}: {{ commonLabels | get(key='alertname') }}";
pub const DEFAULT_MESSAGE_TEMPLATE: &str = "
{{ commonAnnotations | get(key='description', default='no description given') }}
---
{%- for alert in alerts %}
## {{ alert.status }}: {{ alert.labels | get(key='alertname') }}
{{ alert.annotations | get(key='description') }}
---
{% endfor %}
";

mod error;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub ntfy: Ntfy,
    pub topic: Topic,
    pub templates: Option<Templates>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ntfy {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Topic {
    // The default topic used if no other configuration yielded a topic.
    pub default: String,
    // The alert label to extract the target topic from.
    pub label: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Templates {
    pub title: Option<String>,
    pub message: Option<String>,
}

pub fn load(path: &Path) -> Result<Config, Error> {
    let config_file = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_file)?;

    Ok(config)
}

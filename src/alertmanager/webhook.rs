use std::collections::HashMap;

use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Deserialize, Debug)]
pub enum Status {
    #[serde(rename(deserialize = "resolved"))]
    Resolved,
    #[serde(rename(deserialize = "firing"))]
    Firing,
}

// based on https://prometheus.io/docs/alerting/latest/configuration/#webhook_config.
#[derive(Deserialize, Debug)]
pub struct Payload {
    pub version: String,
    #[serde(rename(deserialize = "groupKey"))]
    pub group_key: String,
    #[serde(rename(deserialize = "truncatedAlerts"))]
    pub truncated_alerts: Option<u8>,
    pub status: Status,
    pub receiver: String,
    #[serde(rename(deserialize = "externalURL"))]
    pub external_url: String,
    #[serde(rename(deserialize = "groupLabels"))]
    pub group_labels: Option<HashMap<String, String>>,
    #[serde(rename(deserialize = "commonLabels"))]
    pub common_labels: Option<HashMap<String, String>>,
    #[serde(rename(deserialize = "commonAnnotations"))]
    pub common_annotations: Option<HashMap<String, String>>,
    pub alerts: Vec<Alert>,
}

#[derive(Deserialize, Debug)]
pub struct Alert {
    pub status: Status,
    pub labels: Option<HashMap<String, String>>,
    pub annotations: Option<HashMap<String, String>>,
    #[serde(with = "time::serde::rfc3339")]
    #[serde(rename(deserialize = "startsAt"))]
    pub starts_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[serde(rename(deserialize = "endsAt"))]
    pub ends_at: OffsetDateTime,
    #[serde(rename(deserialize = "generatorURL"))]
    pub generator_url: String,
    pub fingerprint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_deserialize() {
        // payload.json sourced from https://gist.github.com/mobeigi/5a96f326bc06c7d6f283ecb7cb083f2b.
        let payload = serde_json::from_str::<Payload>(include_str!("test/payload.json"));
        if let Err(err) = payload {
            panic!("unexpected error: {}", err)
        }
    }
}

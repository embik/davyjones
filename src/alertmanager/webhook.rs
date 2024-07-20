use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    #[serde(rename = "resolved")]
    Resolved,
    #[serde(rename = "firing")]
    Firing,
}

// based on https://prometheus.io/docs/alerting/latest/configuration/#webhook_config.
#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    pub version: String,
    #[serde(rename = "groupKey")]
    pub group_key: String,
    #[serde(rename = "truncatedAlerts")]
    pub truncated_alerts: Option<u8>,
    pub status: Status,
    pub receiver: String,
    #[serde(rename = "externalURL")]
    pub external_url: String,
    #[serde(rename = "groupLabels")]
    pub group_labels: Option<HashMap<String, String>>,
    #[serde(rename = "commonLabels")]
    pub common_labels: Option<HashMap<String, String>>,
    #[serde(rename = "commonAnnotations")]
    pub common_annotations: Option<HashMap<String, String>>,
    pub alerts: Vec<Alert>,
}

impl Payload {
    pub fn get_common_label(&self, key: &str) -> Option<&String> {
        match &self.common_labels {
            Some(map) => map.get(key),
            None => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

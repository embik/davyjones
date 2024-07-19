use serde::{Deserialize, Serialize};

// Source: https://docs.ntfy.sh/publish/#publish-as-json

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    // Target topic name
    pub topic: String,
    // Message body; set to triggered if empty or not passed
    pub message: Option<String>,
    // Message title
    pub title: Option<String>,
    // List of tags that may or not map to emojis
    pub tags: Option<Vec<String>>,
    // Message priority with 1=min, 3=default and 5=max
    pub priority: Option<u8>,
    // Custom user action buttons for notifications
    // actions,
    // Website opened when notification is clicked
    pub click: Option<url::Url>,
    // URL of an attachment, see attach via URL
    pub attach: Option<url::Url>,
    // Set to true if the message is Markdown-formatted
    pub markdown: Option<bool>,
    // URL to use as notification icon
    pub icon: Option<url::Url>,
    // File name of the attachment
    pub filename: Option<String>,
    // Timestamp or duration for delayed delivery
    pub delay: Option<String>,
    // E-mail address for e-mail notifications
    pub email: Option<String>,
    // Phone number to use for voice call
    pub call: Option<String>,
}

impl Message {
    pub fn new(topic: String) -> Message {
        Message {
            topic,
            message: None,
            title: None,
            tags: None,
            priority: None,
            click: None,
            attach: None,
            markdown: None,
            icon: None,
            filename: None,
            delay: None,
            email: None,
            call: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Message;

    #[test]
    fn test_simple() {
        serde_json::from_str::<Message>(include_str!("test/message.json")).unwrap();
    }
}

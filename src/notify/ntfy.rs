use std::sync::OnceLock;

use reqwest::header::HeaderMap;
use snafu::ResultExt;

use super::*;

#[derive(Deserialize, Serialize)]
pub struct Ntfy {
    url: String,
    token: Option<String>,
    title: Option<String>,
    priority: Option<u8>,
}

impl Debug for Ntfy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ntfy")
            .field("url", &self.url)
            .field("title", &self.title)
            .field("priority", &self.priority)
            .finish()
    }
}

impl Ntfy {
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn new(url: &str, token: Option<&str>, title: Option<&str>, priority: Option<u8>) -> Self {
        Self {
            url: url.to_string(),
            token: token.map(|s| s.to_string()),
            title: title.map(|s| s.to_string()),
            priority,
        }
    }

    fn get_headers(&self) -> Result<&HeaderMap, Error> {
        static HEADER_MAP: OnceLock<reqwest::header::HeaderMap> = OnceLock::new();

        if let Some(header) = HEADER_MAP.get() {
            Ok(header)
        } else {
            let mut headers = reqwest::header::HeaderMap::new();
            if let Some(token) = self.token.as_ref() {
                headers.insert(
                    "Authorization",
                    format!("Bearer {}", token)
                        .parse()
                        .context(HeaderValueSnafu {
                            header: "Authorization".to_string(),
                            value: token.to_string(),
                        })?,
                );
            }
            if let Some(title) = self.title.as_ref() {
                headers.insert(
                    "X-Title",
                    title.parse().context(HeaderValueSnafu {
                        header: "X-Title".to_string(),
                        value: title.to_string(),
                    })?,
                );
            }

            if let Some(priority) = self.priority {
                headers.insert(
                    "X-Priority",
                    priority.to_string().parse().context(HeaderValueSnafu {
                        header: "X-Priority".to_string(),
                        value: priority.to_string(),
                    })?,
                );
            }

            let header = HEADER_MAP.get_or_init(|| headers);
            Ok(header)
        }
    }
}

#[async_trait]
impl Notify for Ntfy {
    async fn send_message(&self, message: &str) -> Result<(), Error> {
        let client = reqwest::Client::new();
        let headers = self.get_headers()?;
        let response = client
            .post(&self.url)
            .headers(headers.clone())
            .body(message.to_string())
            .send()
            .await
            .context(NetworkSnafu)?;

        response.error_for_status().context(NetworkSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use eventsource_client::{Client, SSE};
    use futures_util::StreamExt;

    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        const URL: &str = "https://ntfy.sh/1gxalzBQGTtf99zU";

        // random generator a token
        let token = uuid::Uuid::new_v4().to_string();
        let message = token.clone();
        // sse listener
        let result = tokio::spawn(async move {
            use eventsource_client as es;

            let client = es::ClientBuilder::for_url(&format!("{}/sse", URL))
                .unwrap()
                .build();
            let mut stream = client.stream();
            while let Some(event) = stream.next().await {
                match event {
                    Ok(SSE::Event(event)) => {
                        if event.event_type == "message" {
                            if event.data.contains(token.as_str()) {
                                return Ok(());
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to receive the message: {}", e));
                    }
                    _ => {}
                }
            }

            Err("Failed to receive the message".to_string())
        });

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let ntfy = Ntfy::new(URL, None, Some("title"), Some(1));
        ntfy.send_message(&message).await.unwrap();

        // tokio select wait 3 seconds
        tokio::select! {
            ret = result => {
                assert!(ret.is_ok());
            },
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(3)) => {
                panic!("Failed to receive the message");
            },
        }
    }
}

use std::collections::HashMap;

use snafu::ResultExt;

use super::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct Webhook {
    url: String,
    headers: HashMap<String, String>,
    template: Option<String>,
    method: Method,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
}

impl From<Method> for reqwest::Method {
    fn from(val: Method) -> Self {
        match val {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
        }
    }
}

#[async_trait]
impl Notify for Webhook {
    async fn send_message(&self, message: &str) -> Result<(), Error> {
        let client = reqwest::Client::new();
        let method = self.method.clone();
        let mut request = client
            .request(method.into(), &self.url)
            .body(message.to_string());

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request.send().await.context(NetworkSnafu)?;

        response.error_for_status().context(NetworkSnafu)?;

        Ok(())
    }
}

impl Webhook {
    #[allow(dead_code)]
    pub fn new(
        url: &str,
        headers: HashMap<String, String>,
        template: Option<String>,
        method: Method,
    ) -> Self {
        Webhook {
            url: url.to_string(),
            headers,
            template,
            method,
        }
    }

    pub fn generate_message(&self, under_threshold: bool, rate: f64) -> String {
        self.template
            .to_owned()
            .unwrap_or_default()
            .replace("{under_threshold}", &under_threshold.to_string())
            .replace("{rate}", &rate.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_message() {
        let webhook = Webhook {
            url: "http://example.com".to_string(),
            headers: HashMap::new(),
            template: Some("CNH/CNY is below the warning threshold: {rate}".to_string()),
            method: Method::Post,
        };

        let message = webhook.generate_message(true, 6.5);
        assert_eq!(message, "CNH/CNY is below the warning threshold: 6.5");
    }

    #[test]
    fn test_generate_message_default() {
        let webhook = Webhook {
            url: "http://example.com".to_string(),
            headers: HashMap::new(),
            template: None,
            method: Method::Post,
        };

        let message = webhook.generate_message(true, 6.5);
        assert_eq!(message, "");
    }

    #[test]
    fn test_into_reqwest_method() {
        let method: reqwest::Method = Method::Get.into();
        assert_eq!(method, reqwest::Method::GET);

        let method: reqwest::Method = Method::Post.into();
        assert_eq!(method, reqwest::Method::POST);

        let method: reqwest::Method = Method::Put.into();
        assert_eq!(method, reqwest::Method::PUT);
    }

    #[test]
    fn test_send_message() {
        let mut server = mockito::Server::new();
        let _host = server.host_with_port();
        let url = server.url();

        let _mock = server
            .mock("POST", "/hello")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                "{
                \"message\": \"hello\"
            }",
            )
            .create();

        let webhook = Webhook {
            url: format!("{}{}", url, "/hello"),
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            template: Some(
                "{
                \"message\": \"hello\"
            }"
                .to_string(),
            ),
            method: Method::Post,
        };

        let message = webhook.generate_message(true, 6.5);
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let _result = webhook.send_message(&message).await;
            });
        _mock.assert();
    }
}

use snafu::ResultExt;

use super::*;

#[derive(Deserialize, Serialize)]
pub struct Telegram {
    token: String,
    chat_id: String,
}

impl Debug for Telegram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Telegram")
            .field("chat_id", &self.chat_id)
            .finish()
    }
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
impl Telegram {
    pub fn new(token: &str, chat_id: &str) -> Self {
        Self {
            token: token.to_string(),
            chat_id: chat_id.to_string(),
        }
    }
}

#[async_trait]
impl Notify for Telegram {
    async fn send_message(&self, message: &str) -> Result<(), Error> {
        let token = self.token.clone();
        let chat_id = self.chat_id.clone();

        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "https://api.telegram.org/bot{}/sendMessage",
                token
            ))
            .form(&[("chat_id", chat_id.as_str()), ("text", message)])
            .send()
            .await
            .context(NetworkSnafu)?;

        response.error_for_status().context(NetworkSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        let telegram = Telegram::new(
            "7089203368:AAEn0EX6_V5_cAuUTKRdM1JGYiyXUFtidyo",
            "308930706",
        );
        let message = "Hello, world!";
        telegram.send_message(message).await.unwrap();
    }
}

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::fmt::Debug;

pub mod ntfy;
pub mod telegram;
pub mod webhook;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[enum_dispatch(Notify,Into<NotifyType>)]
pub enum NotifyType {
    Telegram(telegram::Telegram),
    Ntfy(ntfy::Ntfy),
    Webhook(webhook::Webhook),
}

#[async_trait]
#[enum_dispatch]
pub trait Notify {
    async fn send_message(&self, message: &str) -> Result<(), Error>;
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to send the message: {}", source))]
    Network { source: reqwest::Error },

    #[snafu(display("Failed to parse the header value: {}", source))]
    HeaderValue {
        source: reqwest::header::InvalidHeaderValue,
        header: String,
        value: String,
    },
}

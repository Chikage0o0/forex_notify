use std::str::FromStr;

use directories::ProjectDirs;
use notify::{Notify, NotifyType};
use tokio::signal;
use tracing::{debug, info, level_filters::LevelFilter, warn};
mod forex;
mod notify;
mod setting;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let setting = setting::Setting::new(
        "FOREX_NOTIFY_CONFIG",
        ProjectDirs::from("me", "939", "forex_notify"),
    )
    .expect("Failed to load the configuration");

    let log_level = LevelFilter::from_str(&setting.log_level).expect("Invalid log level");
    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("Starting the CNH/CNY forex rate monitor");

    tokio::select! {
        _ = shutdown_signal() => {},
        _ = run_forex(&setting.api_key,setting.warning_threshold,setting.sleeptime,setting.notifiers) => {},
    }

    info!("Shutting down the CNH/CNY forex rate monitor");
}

async fn run_forex(
    api_key: &str,
    warning_threshold: f64,
    sleeptime: u64,
    notifiers: Vec<NotifyType>,
) {
    const CURRENCY1: &str = "USD/CNH";
    const CURRENCY2: &str = "USD/CNY";

    let mut under_threshold = false;
    loop {
        let price1 = forex::get_realtime_price(api_key, CURRENCY1)
            .await
            .inspect_err(|e| {
                warn!("Failed to get the price of {}: {}", CURRENCY1, e);
            });
        let price2 = forex::get_realtime_price(api_key, CURRENCY2)
            .await
            .inspect_err(|e| {
                warn!("Failed to get the price of {}: {}", CURRENCY2, e);
            });
        if price1.is_err() || price2.is_err() {
            tokio::time::sleep(tokio::time::Duration::from_secs(sleeptime)).await;
            continue;
        }
        let cnh_cny = price2.unwrap() / price1.unwrap();

        if cnh_cny < warning_threshold {
            if !under_threshold {
                info!(
                    "CNH/CNY is below the warning threshold: {:.3}",
                    cnh_cny * 100.0
                );
                under_threshold = true;
                let message = format!("CNH/CNY低于预设值，为:{:.3}", (cnh_cny * 100.0));
                for notifier in notifiers.iter() {
                    let ret = if let NotifyType::Webhook(webhook) = notifier {
                        let message = webhook.generate_message(under_threshold, cnh_cny);
                        webhook.send_message(&message).await
                    } else {
                        notifier.send_message(&message).await
                    };

                    let _ = ret
                        .inspect_err(|e| {
                            warn!("Failed to send the message use {:?}: {}", notifier, e);
                        })
                        .inspect(|_| debug!("Successfully sent the message use {:?}", notifier));
                }
            }
        } else if under_threshold {
            info!(
                "CNH/CNY is above the warning threshold: {:.3}",
                cnh_cny * 100.0
            );
            under_threshold = false;
            let message = format!("CNH/CNY高于预设值，为:{:.3}", (cnh_cny * 100.0));
            for notifier in notifiers.iter() {
                let ret = if let NotifyType::Webhook(webhook) = notifier {
                    let message = webhook.generate_message(under_threshold, cnh_cny);
                    webhook.send_message(&message).await
                } else {
                    notifier.send_message(&message).await
                };

                let _ = ret
                    .inspect_err(|e| {
                        warn!("Failed to send the message use {:?}: {}", notifier, e);
                    })
                    .inspect(|_| debug!("Successfully sent the message use {:?}", notifier));
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(sleeptime)).await;
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

use serde::{Deserialize, Serialize};
use std::env;
use warp::Filter;

use chrono::prelude::*;
use chrono_tz::Canada::Atlantic;

fn get_zoned(timestamp: i64) -> String {
    Atlantic
        .from_utc_datetime(&NaiveDateTime::from_timestamp_millis(timestamp).unwrap())
        .format("%d/%m/%Y %H:%M")
        .to_string()
}

#[derive(Deserialize, Serialize)]
struct Message {
    from: String,
    text: String,
    #[serde(alias = "sentStamp")]
    sent_time: i64,
    #[serde(alias = "receivedStamp")]
    recv_time: i64,
    sim: String,
}

impl std::string::ToString for Message {
    fn to_string(&self) -> String {
        format!(
            "{}\n\nFrom: {}\nSim: {}\nSent: {}\nReceived: {}",
            self.text,
            self.from,
            self.sim,
            get_zoned(self.sent_time),
            get_zoned(self.recv_time)
        )
    }
}

#[derive(Deserialize, Serialize)]
struct TelegramMessage {
    chat_id: i64,
    text: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let bot_token: String = env::var("BOT_TOKEN").unwrap();

    let new_message = warp::post()
        .and(warp::path::end())
        .and(warp::body::json())
        .and(warp::any().map(move || bot_token.clone()))
        .and_then(|message: Message, bot_token: String| async move {
            let client = reqwest::Client::new();
            match client
                .post(format!(
                    "https://api.telegram.org/bot{}/{}",
                    bot_token, "sendMessage"
                ))
                .json(&TelegramMessage {
                    chat_id: 627176336,
                    text: message.to_string(),
                })
                .send()
                .await {
                Ok(res) => {
                    if !res.status().is_success() {
                        match res.text().await {
                            Ok(body) => {
                                tracing::error!("Telegram API call was not success, {}", body);
                            }
                            Err(_) => {
                                tracing::error!("Telegram API call was not success, can not extract response text neither");
                            }
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("Can not send a request to Telegram, error: {}", err);
                }
            }
            Ok::<_, std::convert::Infallible>(warp::reply())
        });

    warp::serve(new_message).run(([0, 0, 0, 0], 41893)).await
}

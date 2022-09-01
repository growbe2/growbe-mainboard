use std::sync::mpsc::Receiver;

use futures::{future, pin_mut, Future, TryStreamExt};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::connect_async;
use url::Url;

use crate::comboard::imple::channel::ModuleConfig;

#[derive(Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub topic: String,
    pub data: String,
}

const TOPIC_MODULE_ID: &str = "READ_MODULE_ID";
const TOPIC_MODULES: &str = "READ_SUPPORTED_MODULES";
const TOPIC_MODULE_DATA_PREFIX: &str = "DATA:(.*)";

fn parse_client_addr(config: String) -> Vec<String> {
    return config.split(";").map(|x| x.to_string()).collect();
}

async fn handle_device_loop(url: Url, receiver: Receiver<ModuleConfig>) -> Result<(), ()> {
    let (ws_stream, _) = connect_async(url).await.map_err(|_| ())?;
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let mut module_id: String = "".to_string();
    let mut supported_modules: Vec<String> = vec![];

    let task_read = {
        read.try_for_each(|message| async {
            let data = message.into_data();
            if let Ok(message) = serde_json::from_slice::<WebSocketMessage>(&data) {
                match message.topic.as_str() {
                    TOPIC_MODULE_ID => {}
                    TOPIC_MODULES => {}
                    _ => {
                        // Module data
                    }
                }
            }

            Ok(())
        })
    };

    let task_write = async {
        loop {
            if let Ok(config) = receiver.recv() {
                println!("config to send to ws");
            }
        }
    };

    pin_mut!(task_read, task_write);
    future::select(task_read, task_write).await;

    return Ok(());
}

pub struct WSComboardClient {
    pub config_comboard: crate::comboard::imple::interface::ComboardClientConfig,
}

impl crate::comboard::imple::interface::ComboardClient for WSComboardClient {
    fn run(
        &self,
        receiver_config: Receiver<crate::comboard::imple::channel::ModuleConfig>,
    ) -> tokio::task::JoinHandle<()> {
        let config =
            ("ws://".to_string() + &self.config_comboard.config.clone() + ":5000/live").to_string();

        let url = url::Url::parse(&config).unwrap();

        println!("{:?}", url);

        tokio::spawn(async move {
            loop {
                match handle_device_loop(url.clone(), receiver_config).await {
                    Ok(_) => {}
                    Err(_) => {
                        log::warn!("failed to connect");
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    }
                }
            }
        })
    }
}

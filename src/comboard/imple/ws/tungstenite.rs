use std::sync::mpsc::Receiver;

use futures::TryStreamExt;
use futures_util::StreamExt;
use protobuf::Message;
use serde::{Deserialize, Serialize};
use tokio::select;
use tokio_tungstenite::connect_async;
use url::Url;

use crate::comboard::imple::channel::{comboard_send_state, comboard_send_value};
use crate::protos::module::PhoneAccelerationData;

#[derive(Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub topic: String,
    pub payload: String,
}

const TOPIC_MODULE_ID: &str = "READ_MODULE_ID";
const TOPIC_MODULES: &str = "READ_SUPPORTED_MODULES";

async fn handle_device_loop(url: Url) -> Result<(), ()> {
    let (ws_stream, _) = connect_async(url.clone()).await.map_err(|_| ())?;
    println!("WebSocket handshake has been successfully completed");

    let (write, mut read) = ws_stream.split();

    let mut connected = false;
    let mut module_id: String = "".to_string();
    let mut supported_modules: Vec<String> = vec![];

    loop {
        select! {
            message = read.try_next() => {
                if let Ok(message) = message {
                    let data = message.unwrap().into_data();
                    match serde_json::from_slice::<WebSocketMessage>(&data) {
                        Ok(message) => {
                            match message.topic.as_str() {
                                TOPIC_MODULE_ID => {
								    module_id = message.payload.clone();
                                },
                                TOPIC_MODULES => {
                                    supported_modules = message.payload.split(";").map(|x| x.to_string()).collect();
                                }
                                _ => {

                                }
                            }

                            if !connected && module_id != "" && supported_modules.len() > 0 {
                                connected = true;
                                for (i, module_type) in supported_modules.iter().enumerate() {
                                    let id = module_type.clone() + &module_id;
                                    comboard_send_state("ws".to_string(), url.to_string(), i as i32, id.clone(), true).unwrap();
                                }
                            }
                       }
                       Err(err) => {
                           // Regarde si on est un message protobuf;
                           if module_id.is_empty() {
                               continue;
                           }
                           if data[0] <= 10 {
                                comboard_send_value("ws".to_string(), url.to_string(), data[0] as i32, data[1..data.len()].to_vec()).unwrap();
                           } else {
                               log::error!("error parsing json : {:?}", err);
                           }
                       }
                    }
                } else {
                    log::error!("error try_next websocket");
                    return Err(());
                }
            }
        }
    }
}

pub struct WSComboardClient {
    pub config_comboard: crate::comboard::imple::interface::ComboardClientConfig,
}

impl crate::comboard::imple::interface::ComboardClient for WSComboardClient {
    fn run(
        &self,
        _receiver_config: Receiver<crate::comboard::imple::channel::ModuleConfig>,
    ) -> tokio::task::JoinHandle<()> {
        let config =
            ("ws://".to_string() + &self.config_comboard.config.clone() + ":5000/live").to_string();

        let url = url::Url::parse(&config).unwrap();

        return tokio::spawn(async move {
            loop {
                match handle_device_loop(url.clone()).await {
                    Ok(_) => {}
                    Err(_) => {
                        log::warn!("waiting to retry on websocket client");
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    }
                }
            }
        });
    }
}

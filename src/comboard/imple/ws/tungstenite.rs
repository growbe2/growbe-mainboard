use std::sync::mpsc::Receiver;

use serde::{Deserialize, Serialize};
use url::Url;

use tungstenite::connect;

use crate::{comboard::imple::channel::{comboard_send_state, comboard_send_value}, id::get};

#[derive(Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub topic: String,
    pub payload: String,
}

impl WebSocketMessage {
    fn new(topic: &str, payload: &str) -> Self {
        WebSocketMessage { topic: topic.to_string(), payload: payload.to_string() }
    }
}

const TOPIC_MODULE_ID: &str = "READ_MODULE_ID";
const TOPIC_MODULES: &str = "READ_SUPPORTED_MODULES";

fn send_module_state(module_id: &String, supported_modules: &Vec<String>, url: &Url, state: bool) {
    for (i, module_type) in supported_modules.iter().enumerate() {
        let id = module_type.clone() + module_id;
        comboard_send_state(
            "ws".to_string(),
            url.host_str().unwrap().to_string(),
            i as i32,
            id.clone(),
            state,
        )
        .unwrap();
    }
}

fn handle_device_loop(
    url: Url,
    receiver_config: &Receiver<crate::comboard::imple::channel::ModuleConfig>,
) -> Result<(), ()> {
    log::info!("trying to connect to websocket");
    let (mut ws_stream, _) = connect(url.clone()).map_err(|_| ())?;

    log::info!(
        "WebSocket handshake has been successfully completed {:?}",
        url.to_string()
    );

    let mut connected = false;
    let mut module_id: String = "".to_string();
    let mut supported_modules: Vec<String> = vec![];


    let msg = WebSocketMessage::new("MAINBOARD_ID", &get());

    if let Err(err) = ws_stream.write_message(tungstenite::Message::Text(serde_json::to_string(&msg).unwrap())) {
        log::error!("failed to send mainboard id to module : {:?}", err);
        return Err(());
    }

    loop {
        match ws_stream.read_message() {
            Ok(data) => {
            let data = data.into_data();
            match serde_json::from_slice::<WebSocketMessage>(&data) {
                Ok(message) => {
                    match message.topic.as_str() {
                        TOPIC_MODULE_ID => {
                            module_id = message.payload.clone();
                        }
                        TOPIC_MODULES => {
                            supported_modules =
                                message.payload.split(";").map(|x| x.to_string()).collect();
                        }
                        _ => {}
                    }

                    if !connected && module_id != "" && supported_modules.len() > 0 {
                        connected = true;
                        send_module_state(&module_id, &supported_modules, &url, true);
                    }
                }
                Err(err) => {
                    // Regarde si on est un message protobuf;
                    if module_id.is_empty() {
                        continue;
                    }
                    if data[0] <= 10 {
                        comboard_send_value(
                            "ws".to_string(),
                            url.host_str().unwrap().to_string(),
                            data[0] as i32,
                            data[1..data.len()].to_vec(),
                        )
                        .unwrap();
                    } else {
                        log::error!("error parsing json : {:?}", err);
                    }
                }
            }

            }
            Err(err) => {
                log::debug!("error try_next websocket {:?}", err.to_string());
                if supported_modules.len() > 0 {
                    send_module_state(&module_id, &supported_modules, &url, false);
                }
                return Err(());
            }
        }

        if let Ok(value) = receiver_config.try_recv() {
            // TODO fix size and copy into buffer
            let mut data = vec![value.port as u8];
            data.append(&mut value.data.clone());
        
            if let Err(_) = ws_stream.write_message(tungstenite::Message::Binary(data)) {
                println!("Failed to write message");
            } else {
                log::warn!("sending config to websocket successfuly");
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
        receiver: Receiver<crate::comboard::imple::channel::ModuleConfig>,
    ) -> tokio::task::JoinHandle<Result<(), ()>> {
        let config =
            ("ws://".to_string() + &self.config_comboard.config.clone() + ":5000/live").to_string();

        let url = url::Url::parse(&config).unwrap();

        let receiver = receiver;

        return tokio::spawn(async move {
            loop {
                match handle_device_loop(url.clone(), &receiver) {
                    Ok(_) => {}
                    Err(_) => {
                        //log::debug!("waiting to connect on {}", url.to_string());
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
            }
        });
    }
}

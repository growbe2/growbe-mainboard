use tokio::sync::mpsc::{Receiver, Sender};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    comboard::imple::interface::{ModuleValueValidationEvent, ModuleStateChangeEvent}, mainboardstate::error::MainboardError,
    modulestate::interface::ModuleMsg,
};

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub topic: String,
    pub payload: String,
}

impl WebSocketMessage {
    fn new(topic: &str, payload: &str) -> Self {
        WebSocketMessage {
            topic: topic.to_string(),
            payload: payload.to_string(),
        }
    }
}

#[derive(Default)]
pub struct Context {
    pub connected: bool,
    pub module_id: String,
    pub supported_modules: Vec<String>,
}

const TOPIC_MODULE_ID: &str = "READ_MODULE_ID";
const TOPIC_MODULES: &str = "READ_SUPPORTED_MODULES";

async fn send_module_state(
    sender_module: &Sender<ModuleMsg>,
    module_id: &String,
    supported_modules: &Vec<String>,
    url: &Url,
    state: bool
) -> Result<(), MainboardError> {
    for (i, module_type) in supported_modules.iter().enumerate() {
        let id = module_type.clone() + module_id;
        sender_module.send(
            ModuleMsg::State(ModuleStateChangeEvent{
               board: "ws".into(),
               board_addr: url.host_str().unwrap().into(),
               id: id.clone(),
               port: i as i32,
               state,
            })
        ).await?;
    }
    Ok(())
}

async fn on_data(
    url: &Url,
    ctx: &mut Context,
    sender_module: &Sender<ModuleMsg>,
    data: Message,
) -> Result<(), MainboardError> {
    let data = data.into_data();
    match serde_json::from_slice::<WebSocketMessage>(&data) {
        Ok(message) => {
            match message.topic.as_str() {
                TOPIC_MODULE_ID => {
                    ctx.module_id = message.payload.clone();
                    log::info!("module_id {:?}", ctx.module_id);
                }
                TOPIC_MODULES => {
                    ctx.supported_modules =
                        message.payload.split(";").map(|x| x.to_string()).collect();

                    log::info!("supportedmodule_id {:?}", ctx.module_id);
                }
                _ => {
                    log::info!("DADADADAD");
                }
            }

            if !ctx.connected && ctx.module_id != "" && ctx.supported_modules.len() > 0 {
                ctx.connected = true;
                send_module_state(&sender_module, &ctx.module_id, &ctx.supported_modules, &url, true).await?;
            }
        }
        Err(_err) => {
            // Regarde si on est un message protobuf;
            if ctx.module_id.is_empty() {
                return Ok(());
            }
            if data.len() > 0 && data[0] <= 10 {
                sender_module
                    .send(ModuleMsg::Value(ModuleValueValidationEvent {
                        board: "ws".into(),
                        board_addr: url.host_str().unwrap().into(),
                        port: data[0] as i32,
                        buffer: data[1..data.len()].to_vec(),
                    }))
                    .await?;
            }
        }
    }
    return Ok(());
}

async fn handle_device_loop(
    url: Url,
    sender_module: &Sender<ModuleMsg>,
    receiver_config: &mut Receiver<crate::comboard::imple::channel::ModuleConfig>,
) -> Result<(), MainboardError> {
    let (mut ws_stream, _) = connect_async(url.clone()).await?;

    let mut ctx = Context::default();

    println!("WebSocket handshake has been successfully completed");

    loop {
        tokio::select! {
            Some(item) = ws_stream.next() => {
                match item {
                    Ok(message) => {
                        on_data(&url, &mut ctx, &sender_module, message).await?;
                    },
                    Err(err) => {
                        log::error!("ws error : {:?}", err);
                        if ctx.connected == true {
                            send_module_state(&sender_module, &ctx.module_id, &ctx.supported_modules, &url, false).await?;
                            return Err(MainboardError { message: "disconnection from target".into() })
                        }
                    }
                }
            },
            Some(value) = receiver_config.recv() => {
                let mut data = vec![value.port as u8];
                data.append(&mut value.data.clone());

                if let Err(_) = ws_stream.send(Message::Binary(data)).await {
                    log::error!("Failed to write message");
                } else {
                    log::warn!("sending config to websocket successfuly");
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
        &mut self,
        sender_module: Sender<ModuleMsg>,
        mut receiver: Receiver<crate::comboard::imple::channel::ModuleConfig>,
    ) -> tokio::task::JoinHandle<Result<(), ()>> {
        let config =
            ("ws://".to_string() + &self.config_comboard.config.clone() + ":5000/live").to_string();

        let url = url::Url::parse(&config).unwrap();

        return tokio::spawn(async move {
            loop {
                match handle_device_loop(url.clone(), &sender_module, &mut receiver).await {
                    Ok(_) => {}
                    Err(_) => {
                        log::debug!("waiting to connect on {}", url.to_string());
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
            }
        });
    }
}

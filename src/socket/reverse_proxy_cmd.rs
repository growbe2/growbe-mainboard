use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;
use url::Url;

use crate::protos::board::{GrowbeCommand, MQTTConfig};
use crate::socket::ss::SenderPayloadData;
use crate::socket::{TaskContext, MAPPING_MODULES, MQTT_HANDLES};
use crate::{
    mainboardstate::error::MainboardError, modulestate::interface::ModuleMsg,
    protos::virt::VirtualScenarioItem,
};
use protobuf::Message;

use super::handle_incomming_message;
use super::ss::SenderPayload;
use futures_util::{SinkExt, StreamExt};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ReverseProxyConf {
    pub url: String,
}

async fn handle_proxy_loop(
    url: &Url,
    sender_socket: &Sender<SenderPayload>,
    sender_virt: &Sender<Vec<VirtualScenarioItem>>,
    sender_module: &Sender<ModuleMsg>,
) -> Result<(), MainboardError> {
    let (mut ws_stream, _) = connect_async(url.clone()).await?;

    log::info!("connected to reverse proxy");
    let ctx = TaskContext {
        sender_virt: sender_virt.clone(),
        sender_module: sender_module.clone(),
    };

    loop {
        tokio::select! {
         Some(item) = ws_stream.next() => {
            match item {
                Ok(message) => {
                    if !message.is_binary() {
                        continue;
                    }
                    let data = message.into_data();
                    let message = GrowbeCommand::parse_from_bytes(&data).unwrap();

                    let data = std::sync::Arc::new(message.payload);

                    let messages = handle_incomming_message(
                        &MQTT_HANDLES,
                        &MAPPING_MODULES,
                        &ctx,
                        message.topic,
                        data,
                    )
                    .await;
                    if messages.len() > 1 {
                        if let Some(message) = messages.first() {
                            // todo should not need to copy and take ownership instead
                            sender_socket
                                .send((
                                    message.0.clone(),
                                    SenderPayloadData::Buffer(message.1.clone()),
                                ))
                                .await?;
                        }
                    }
                    if let Some(message) = messages.last() {
                        let mut response = GrowbeCommand::new();
                        response.topic = message.0.clone();
                        response.payload = message.1.clone();

                        let data = response.write_to_bytes().unwrap();
                        ws_stream
                            .send(tokio_tungstenite::tungstenite::Message::Binary(data))
                            .await
                            .unwrap();
                    } else {
                        println!("ERROR NOTHING TO RESPOND");
                    }
                }
                Err(err) => {
                    log::error!("ws error: {}", err);
                    return Err(MainboardError::from_error(err.to_string()));
                }
            }
        },
        _ = tokio::time::sleep(std::time::Duration::from_secs(45)) => {
            ws_stream.send(tokio_tungstenite::tungstenite::Message::Ping(vec![1,9,9,5])).await.unwrap();
        }
         }
    }
}

pub fn task_reverse_proxy_cmd(
    config: ReverseProxyConf,
    sender_socket: Sender<SenderPayload>,
    sender_virt: Sender<Vec<VirtualScenarioItem>>,
    sender_module: Sender<ModuleMsg>,
) -> tokio::task::JoinHandle<()> {
    return tokio::spawn(async move {
        let config = format!("{}/mainboard/{}", config.url, growbe_shared::id::get());

        let url = url::Url::parse(&config).unwrap();

        loop {
            match handle_proxy_loop(&url, &sender_socket, &sender_virt, &sender_module).await {
                Ok(_) => {}
                Err(e) => {
                    log::debug!("waiting to connect on {} {:?}", url.to_string(), e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });
}

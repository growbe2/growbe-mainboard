use std::time::Duration;

use tokio::{net::TcpListener, sync::mpsc::Receiver};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    channel::{ModuleConfig, ModuleConfigChannelManager},
    config::CONFIG,
    modules::{get_module_client, Module},
    ws::{WebSocketMessage, MSG_READ_MODULE_ID, MSG_READ_SUPPORTED_MODULES},
};
use futures_util::{future, pin_mut, stream::TryStreamExt, SinkExt, StreamExt};

mod channel;
mod config;
mod id;
mod modules;
mod protos;
mod ws;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    if let Some(()) = growbe_shared::cmd::handle_command_line_arguments() {
        return;
    }

    growbe_shared::logger::setup_log(&config::CONFIG.logger);

    log::info!("starting module with id {}", id::get());
    log::info!("binding tcp to {}", CONFIG.server.to_string());

    let server = match TcpListener::bind(CONFIG.server.to_string()).await {
        Ok(e) => e,
        Err(err) => {
            log::error!("TcpListener::bind : {:?}", err);
            return;
        }
    };

    let (sender_module, receiver_module) = tokio::sync::mpsc::channel(10);

    let modules: Vec<Module> = CONFIG.modules.clone();
    let mut channel_manager = ModuleConfigChannelManager::new(sender_module);

    for d in modules.iter() {
        let client = get_module_client(&d.name).unwrap();
        let (receiver, sender) = channel_manager.create_channel(d.port);
        client.run(receiver, sender);
    }

    let receiver_module = tokio::sync::Mutex::<_>::new(receiver_module);

    while let Ok((raw_stream, addr)) = server.accept().await {
        let ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
            Ok(ws_stream) => ws_stream,
            Err(err) => {
                log::error!("{:?}", err);
                continue;
            }
        };

        println!("got connection");

        // IF ALREADY CONNECT BLOCK NEW CONNECTION

        let (mut outgoing, incoming) = ws_stream.split();

        let msg = WebSocketMessage {
            payload: id::get().to_string(),
            topic: MSG_READ_MODULE_ID.to_string(),
        };
        let msg_vec = serde_json::to_vec(&msg).unwrap();
        outgoing.send(Message::binary(msg_vec)).await.unwrap();

        let msg = WebSocketMessage {
            payload: modules
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>()
                .join(";"),
            topic: MSG_READ_SUPPORTED_MODULES.to_string(),
        };
        let msg_vec = serde_json::to_vec(&msg).unwrap();
        outgoing.send(Message::binary(msg_vec)).await.unwrap();

        let broadcast_incoming = incoming.try_for_each(|msg| {
            if msg.is_binary() {
                println!("Received a message from {}", addr);
                let data = msg.into_data();
                let port = data[0] as i32;
                let config_data = data[1..].to_vec();

                channel_manager
                    .get_reference()
                    .send(
                        port,
                        ModuleConfig {
                            port,
                            data: config_data,
                        },
                    )
                    .unwrap();
            }
            future::ok(())
        });

        let receive_from_others = {
            async {
                let modules = modules.clone();
                loop {
                    if let Err(_) = outgoing.send(Message::Ping(vec![])).await {
                        break;
                    }
                    if let Ok(mut value) = receiver_module.lock().await.try_recv()
                    {
                        let mut index: i32 = -1;
                        for (i, val) in modules.iter().enumerate() {
                            if val.name == value.0 {
                                index = i as i32;
                                break;
                            }
                        }
                        if index > -1 {
                            value.1.insert(0, index as u8);
                            outgoing.send(Message::Binary(value.1)).await.unwrap();
                        }
                    }
                }
            }
        };

        pin_mut!(broadcast_incoming, receive_from_others);
        future::select(broadcast_incoming, receive_from_others).await;

        println!("disconnected ????");
    }
}

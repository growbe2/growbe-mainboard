
use std::{time::Duration, process::id};

use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

use crate::{config::CONFIG, modules::{Module, get_module_client}, channel::{ModuleConfigChannelManager, ModuleConfig}, ws::{WebSocketMessage, MSG_READ_MODULE_ID, MSG_READ_SUPPORTED_MODULES}};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt, SinkExt};

mod config;
mod id;
mod logger;
mod modules;
mod ws;
mod channel;
mod protos;


#[tokio::main(flavor = "multi_thread")]
async fn main() {
    logger::setup_log();

    log::info!("starting module with id {}", id::get());

    let server = match TcpListener::bind(CONFIG.server.to_string()).await {
        Ok(e) => e,
        Err(err) => {
            log::error!("{:?}", err);
            return;
        }
    };

	let modules: Vec<Module> = CONFIG.modules.clone();
	let mut channel_manager = ModuleConfigChannelManager::new();

	for d in modules.iter() {
		let client = get_module_client(&d.name).unwrap();
		let receiver = channel_manager.create_channel(d.port);
		client.run(receiver);
	}

	while let Ok((raw_stream, addr)) = server.accept().await {
		let ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
			Ok(ws_stream) => ws_stream,
			Err(err) => { log::error!("{:?}", err); continue; }
		};

		println!("got connection");


		let (mut outgoing, incoming) = ws_stream.split();

		let msg = WebSocketMessage{payload: id::get().to_string(), topic: MSG_READ_MODULE_ID.to_string() };
		let msg_vec = serde_json::to_vec(&msg).unwrap();
		outgoing.send(Message::binary(msg_vec)).await.unwrap();

		let msg = WebSocketMessage{payload: modules.iter().map(|x| x.name.clone()).collect::<Vec<String>>().join(";"), topic: MSG_READ_SUPPORTED_MODULES.to_string() };
		let msg_vec = serde_json::to_vec(&msg).unwrap();
		outgoing.send(Message::binary(msg_vec)).await.unwrap();


		let broadcast_incoming = incoming.try_for_each(|msg| {
			if msg.is_binary() {
				println!("Received a message from {}", addr);
				let data = msg.into_data();
				let port = data[0] as i32;
				let config_data = data[1..].to_vec();

				channel_manager.get_reference().send(port, ModuleConfig{ port, data: config_data}).unwrap();
			}
			future::ok(())
		});


		let receive_from_others = {
			let modules = modules.clone();
			async move {
				loop {
					if let Err(_) = outgoing.send(Message::Ping(vec![])).await {
						break;
					}
					if let Ok(mut value) = channel::CHANNEL_VALUE.1.lock().unwrap().recv_timeout(Duration::from_millis(1)) {
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

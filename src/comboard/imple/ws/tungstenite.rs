use std::sync::mpsc::Receiver;

use futures::TryStreamExt;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::connect_async;
use url::Url;

fn parse_client_addr(config: String) -> Vec<String> {
    return config.split(";").map(|x| x.to_string()).collect();
}

async fn handle_device_loop(url: Url) -> Result<(), ()> {
    let (ws_stream, _) = connect_async(url)
        .await
		.map_err(|_| ())?;
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    //let ws_to_stdout = {
    read.try_for_each(|message| async {
		let data = message.into_data();

        // Send data to comboard manager
		tokio::io::stdout().write_all(&data).await.unwrap();
		Ok(())
    })
    .await.map_err(|_| ())?;
    //};

	Ok(())
}

pub struct WSComboardClient {
    pub config_comboard: crate::comboard::imple::interface::ComboardClientConfig,
}

impl crate::comboard::imple::interface::ComboardClient for WSComboardClient {
    fn run(&self, receiver_config: Receiver<crate::comboard::imple::channel::ModuleConfig>) -> tokio::task::JoinHandle<()> {
        let config =
            ("ws://".to_string() + &self.config_comboard.config.clone() + ":5000/live").to_string();

        let url = url::Url::parse(&config).unwrap();

        println!("{:?}", url);

        return tokio::spawn(async move {
            loop {
				match handle_device_loop(url.clone()).await {
					Ok(_) => {},
					Err(_) => {
						log::warn!("failed to connect");
						tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
					}
				}
			}
        });
    }
}

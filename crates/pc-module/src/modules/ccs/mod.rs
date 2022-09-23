use std::time::Duration;

use protobuf::Message;
use tokio::process::{Command, Child};

use crate::{channel::CHANNEL_VALUE, protos::module::{ComputerStreamingData, ComputerStreamingConfig}};

use super::ModuleClient;



pub struct StreamingModule {}


impl StreamingModule {
	pub fn new() -> Self {
		StreamingModule {  }
	}
}


impl ModuleClient for StreamingModule {
	fn run(&self, receiver_config: std::sync::mpsc::Receiver<crate::channel::ModuleConfig>) -> tokio::task::JoinHandle<Result<(), ()>> {
		return tokio::spawn(async move {
			log::info!("start modle");

			let mut v = ComputerStreamingData::new();

			let mut previousConfig = ComputerStreamingConfig::new();
			let mut config = ComputerStreamingConfig::new();

			let mut child: Option<Child> = None;

			loop {
				if let Ok(msg) = receiver_config.try_recv() {
					previousConfig = config.clone();
					config = match ComputerStreamingConfig::parse_from_bytes(&msg.data) {
						Ok(config) => config,
						Err(err) => { log::error!("error parsing config : {:?}", err); continue; }
					};

					let command = Command::new("ffmpeg")
						.args("-thread_queue_size 1024 -f alsa -guess_layout_max 0 -thread_queue_size 512 -f v4l2 -video_size 800x600 -framerate 25 -i /dev/video0 -c:v libx264 -pix_fmt yuv420p -preset veryfast -g 50 -b:v 2500k -maxrate 2500k -bufsize 7500k -acodec aac -b:a 32k -f flv".split(" "))
						.arg("rtmp://stream.dev.growbe.ca/live/568277-p3?sign=1679535859637-530abb18cfc125b0f354e58056a4bb51")
						.spawn();

					child = Some(command.unwrap());
				}

				if let Some(child_mut) = child.as_mut() {
					if let Some(output) = child_mut.stdout.take() {
						println!("{:?}", output);
					}
					if let Ok(exit_status) = child_mut.try_wait() {
						if let Some(exit_status) = exit_status {
							println!("exit with: {:?}", exit_status);
							child = None;
						}
					}
				}

				//CHANNEL_VALUE.0.lock().unwrap().send(("CCS".to_string(), v.write_to_bytes().unwrap())).unwrap();
			}
		});
	}
}
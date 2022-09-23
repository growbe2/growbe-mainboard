
use protobuf::{Message, Clear};
use tokio::{process::{Command, Child}};

use crate::{channel::CHANNEL_VALUE, protos::module::{ComputerStreamingData, ComputerStreamingConfig, PhoneStreamingStatus}};

use super::ModuleClient;



pub struct StreamingModule {}


impl StreamingModule {
	pub fn new() -> Self {
		StreamingModule {  }
	}
}

// frame=  140 fps=9.1 q=18.0 size=    4061kB time=00:00:13.20 bitrate=2520.2kbits/s speed=0.862x

impl ModuleClient for StreamingModule {
	fn run(&self, receiver_config: std::sync::mpsc::Receiver<crate::channel::ModuleConfig>) -> tokio::task::JoinHandle<Result<(), ()>> {
		return tokio::spawn(async move {
			log::info!("start modle");

			let mut v = ComputerStreamingData::new();

			let mut config = ComputerStreamingConfig::new();

			let mut child: Option<Child> = None;

			loop {
				if let Ok(msg) = receiver_config.try_recv() {
					config = match ComputerStreamingConfig::parse_from_bytes(&msg.data) {
						Ok(config) => config,
						Err(err) => { log::error!("error parsing config : {:?}", err); continue; }
					};

					if let Some(child) = child.as_mut() {
						child.start_kill().unwrap();
					}

					child = None;
					v.clear();

					if config.activated {
						let mut args = config.arguments.to_vec();
						args.push(config.url);
						let command = Command::new("ffmpeg")
							.args(format!("-hide_banner -loglevel error -thread_queue_size 1024 -f alsa -guess_layout_max 0 -thread_queue_size 512 -f v4l2 -video_size 800x600 -framerate 25 -i {} -c:v libx264 -pix_fmt yuv420p -preset veryfast -g 50 -b:v 2500k -maxrate 2500k -bufsize 7500k -acodec aac -b:a 32k -f flv", config.camera).split(" "))
							.args(args)
							.spawn();

						child = Some(command.unwrap());
						v.set_status(PhoneStreamingStatus::RUNNING);
					} else {
						v.set_status(PhoneStreamingStatus::STOPPED);
					}

					CHANNEL_VALUE.0.lock().unwrap().send(("CCS".to_string(), v.write_to_bytes().unwrap())).unwrap();
				}

				if let Some(child_mut) = child.as_mut() {
					if let Some(output) = child_mut.stdout.take() {
						println!("CAC: {:?}", output);
					}
					if let Ok(exit_status) = child_mut.try_wait() {
						if let Some(exit_status) = exit_status {
							println!("exit with: {:?}", exit_status);
							child = None;
							v.set_status(PhoneStreamingStatus::ERROR);
							v.set_error(format!("error code : {}", exit_status));

							CHANNEL_VALUE.0.lock().unwrap().send(("CCS".to_string(), v.write_to_bytes().unwrap())).unwrap();
						}
					}
				}

			}
		});
	}
}
use std::sync::mpsc::Receiver;

use serde::{Deserialize, Serialize};

use self::{ccs::StreamingModule, css::SystemStatsModule};

pub mod ccs;
pub mod css;


#[derive(Serialize, Deserialize, Clone)]
pub struct Module {
	pub name: String,
	pub port: i32,
}

pub trait ModuleClient {
	fn run(&self, receiver_config: Receiver<super::channel::ModuleConfig>) -> tokio::task::JoinHandle<Result<(), ()>>;
}

pub fn get_module_client(name: &str) -> Option<Box<dyn ModuleClient>> {
	match name {
		"CCS" => Some(Box::new(StreamingModule::new())),
		"CSS" => Some(Box::new(SystemStatsModule::new())),
		_ => None,
	}
}

use serde::{Deserialize, Serialize};

use super::update::{UpdateConfig, get_default_update_config};
use crate::server::http::get_default_server_config;
use crate::logger::{LoggerConfig, default_logger};

lazy_static::lazy_static! {
	pub static ref CONFIG: MainboardProcessConfig = {
		let args: Vec<String> = std::env::args().collect();
	    if args.len() == 1 ||  args[1].is_empty() {
	        panic!("config not passed as args[1]");
	    }
	    return get(&args[1]).unwrap();
	};
}

#[derive(Serialize, Deserialize)]
pub struct MainboardProcessConfig {
    #[serde(default)] 
	pub id: String,
	pub mqtt: crate::socket::mqtt::CloudMQTTConfig,
	pub comboard: crate::comboard::config::ComboardConfig,
    #[serde(default = "get_default_server_config")] 
	pub server: crate::server::http::HttpServerConfig,
	#[serde(default = "default_logger")]
	pub logger: LoggerConfig,
	#[serde(default = "get_default_update_config")]
	pub update: UpdateConfig,
}


pub fn get(config: &String) -> Result<MainboardProcessConfig, serde_json::Error>  {
    let file = std::fs::File::open(config).expect("Error open file");
    let scenario: MainboardProcessConfig = serde_json::from_reader(file)?;
    Ok(scenario)
}


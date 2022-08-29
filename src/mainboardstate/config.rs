
use serde::{Deserialize, Serialize};

use super::update::{UpdateConfig, get_default_update_config};
use crate::server::http::get_default_server_config;
use crate::logger::{LoggerConfig, default_logger};

use crate::protos::{board::{MainboardConfig, MQTTConfig, HttpServerConfig, LoggerConfig as ProtoLoggerConfig, UpdaterConfig, ComboardConfig}};

impl crate::modulestate::interface::ModuleValue for crate::protos::board::MainboardConfig {}
impl crate::modulestate::interface::ModuleValueParsable for crate::protos::board::MainboardConfig {}


fn get_config_path() -> String {
	let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 ||  args[1].is_empty() {
        panic!("config not passed as args[1]");
    }
	return args[1].clone()
}

fn get_default_comboards() -> Vec<crate::comboard::config::ComboardConfig> {
	return vec![];
}

fn get_default_comboard() -> crate::comboard::config::ComboardConfig {
	return crate::comboard::config::ComboardConfig {
		config: "".to_string(),
		imple: "".to_string()
	}
}

lazy_static::lazy_static! {
	pub static ref CONFIG: MainboardProcessConfig = {
	    return get(&get_config_path()).unwrap();
	};
}

#[derive(Serialize, Deserialize)]
pub struct MainboardProcessConfig {
    #[serde(default)] 
	pub id: String,
	pub mqtt: crate::socket::mqtt::CloudMQTTConfig,
	// DEPRECATED , will be replace with comboards array.
    #[serde(default = "get_default_comboard")] 
	pub comboard: crate::comboard::config::ComboardConfig,
    #[serde(default = "get_default_comboards")] 
	pub comboards: Vec<crate::comboard::config::ComboardConfig>,
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


pub fn rewrite_configuration(config: MainboardConfig) -> () {
	let config_path = get_config_path();

	let config_json = MainboardProcessConfig{
		id: config.id,
		mqtt: crate::socket::mqtt::CloudMQTTConfig{
			port: config.mqtt.get_ref().port as u16,
			url: config.mqtt.get_ref().url.clone()
		},
		comboard: crate::comboard::config::ComboardConfig{
			config: config.comboard.get_ref().config.clone(),
			imple: config.comboard.get_ref().imple.clone()
		},
		comboards: vec![],
		server: crate::server::http::HttpServerConfig{
			addr: config.server.get_ref().addr.clone(),
			port: config.server.get_ref().port as u16,
		},
		logger: LoggerConfig{
			target: config.logger.get_ref().target.clone(),
			systemd: true,
		},
		update: UpdateConfig{
			autoupdate: config.update.get_ref().autoupdate,
			channel: config.update.get_ref().channel.clone(),
			reboot: config.update.get_ref().reboot,
		}
	};

	let d = serde_json::to_string_pretty(&config_json).unwrap();

	std::fs::write(config_path, d).unwrap();


	tokio::task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        crate::plateform::restart::restart_process();
    });
	tokio::task::spawn( async {});
}

pub fn get_configuration_proto() -> MainboardConfig {
    let mut config = MainboardConfig::new();

    let mut mqtt = MQTTConfig::new();
	mqtt.port = CONFIG.mqtt.port as i32;
	mqtt.url = CONFIG.mqtt.url.clone();

    let mut comboard = ComboardConfig::new();
	comboard.config = CONFIG.comboard.config.clone();
	comboard.imple = CONFIG.comboard.imple.clone();

    let mut logger = ProtoLoggerConfig::new();
	logger.target = CONFIG.logger.target.clone();

    let mut http = HttpServerConfig::new();
	http.addr = CONFIG.server.addr.clone();
	http.port = CONFIG.server.port as i32;

    let mut update = UpdaterConfig::new();
	update.autoupdate = CONFIG.update.autoupdate;
	update.channel = CONFIG.update.channel.clone();
	update.reboot = CONFIG.update.reboot;


    config.set_id(CONFIG.id.clone());
    config.set_mqtt(mqtt);
    config.set_logger(logger);
    config.set_server(http);
    config.set_comboard(comboard);
    config.set_update(update);


    return config;
}

use serde::{Deserialize, Serialize};

use crate::modules::Module;
use crate::ws::{default_server_config, WSServerConfig};
use growbe_shared::logger::LoggerConfig;

lazy_static::lazy_static! {
    pub static ref CONFIG: MainboardProcessConfig = {
        return match get(&growbe_shared::config::get_config_path()) {
            Ok(config) => config,
            Err(_) => {
                return MainboardProcessConfig::default();
            }
        }
    };
}

pub fn default_logger() -> LoggerConfig {
    return LoggerConfig {
        target: String::from("growbe_pc_module=warn"),
        systemd: false,
    };
}

#[derive(Serialize, Deserialize, Default)]
pub struct MainboardProcessConfig {
    #[serde(default)]
    pub id: String,
    #[serde(default = "default_logger")]
    pub logger: LoggerConfig,
    #[serde(default = "default_server_config")]
    pub server: WSServerConfig,
    pub modules: Vec<Module>,
}

pub fn get(config: &String) -> Result<MainboardProcessConfig, ()> {
    let file = std::fs::File::open(config).map_err(|_| ())?;
    let scenario: MainboardProcessConfig = serde_json::from_reader(file).map_err(|_| ())?;
    Ok(scenario)
}

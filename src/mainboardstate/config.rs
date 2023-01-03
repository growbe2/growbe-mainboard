use protobuf::RepeatedField;
use serde::{Deserialize, Serialize};

use super::error::MainboardError;
use super::update::{get_default_update_config, UpdateConfig};
use crate::plateform::external::is_systemd;
use crate::server::config::get_default_server_config;
use crate::socket::http::{get_default_api_config, APIConfig};

use crate::protos::board::{
    ApiConfig, ComboardConfig, HttpServerConfig, LoggerConfig as ProtoLoggerConfig, MQTTConfig,
    MainboardConfig, UpdaterConfig,
};
use crate::socket::reverse_proxy_cmd::ReverseProxyConf;
use crate::socket::ss::{SenderPayload, SenderSocket};

impl crate::modulestate::interface::ModuleValue for crate::protos::board::MainboardConfig {}
impl crate::modulestate::interface::ModuleValueParsable for crate::protos::board::MainboardConfig {}

pub fn default_logger() -> growbe_shared::logger::LoggerConfig {
    return growbe_shared::logger::LoggerConfig {
        target: String::from("growbe_mainboard=warn"),
        systemd: false,
    };
}

fn get_default_comboards() -> Vec<crate::comboard::config::ComboardConfig> {
    return vec![];
}

fn get_default_comboard() -> crate::comboard::config::ComboardConfig {
    return crate::comboard::config::ComboardConfig {
        config: "".to_string(),
        imple: "".to_string(),
    };
}

fn get_default_reverse_config() -> ReverseProxyConf {
    return ReverseProxyConf {
        url: "wss://proxy.growbe.ca".into(),
    };
}

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

#[derive(Serialize, Deserialize, Default)]
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
    pub server: crate::server::config::HttpServerConfig,
    #[serde(default = "default_logger")]
    pub logger: growbe_shared::logger::LoggerConfig,
    #[serde(default = "get_default_update_config")]
    pub update: UpdateConfig,
    #[serde(default = "get_default_api_config")]
    pub api: APIConfig,
    #[serde(default = "get_default_reverse_config")]
    pub proxy: ReverseProxyConf,
}

pub fn get(config: &String) -> Result<MainboardProcessConfig, ()> {
    let file = std::fs::File::open(config).map_err(|_| ())?;
    let scenario: MainboardProcessConfig = serde_json::from_reader(file).map_err(|_| ())?;
    Ok(scenario)
}

pub async fn rewrite_configuration(config: MainboardConfig) -> Result<(), MainboardError> {
    let config_path = growbe_shared::config::get_config_path();

    let config_json = MainboardProcessConfig {
        id: config.id,
        mqtt: crate::socket::mqtt::CloudMQTTConfig {
            port: config.mqtt.get_ref().port as u16,
            url: config.mqtt.get_ref().url.clone(),
        },
        comboard: crate::comboard::config::ComboardConfig::default(),
        comboards: config
            .comboards
            .iter()
            .map(|x| crate::comboard::config::ComboardConfig {
                config: x.config.clone(),
                imple: x.imple.clone(),
                ..Default::default()
            })
            .collect(),
        server: crate::server::config::HttpServerConfig {
            addr: config.server.get_ref().addr.clone(),
            port: config.server.get_ref().port as u16,
        },
        logger: growbe_shared::logger::LoggerConfig {
            target: config.logger.get_ref().target.clone(),
            systemd: true,
        },
        update: UpdateConfig {
            autoupdate: config.update.get_ref().autoupdate,
            channel: config.update.get_ref().channel.clone(),
            reboot: config.update.get_ref().reboot,
        },
        api: APIConfig {
            url: config.api.get_ref().url.clone(),
        },
        proxy: ReverseProxyConf {
            url: config.proxy.get_ref().url.clone(),
        },
    };

    let d = serde_json::to_string_pretty(&config_json).unwrap();

    tokio::fs::write(config_path, d).await?;

    if is_systemd() {
        tokio::task::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            if let Err(err) = crate::plateform::restart::restart_process() {
                log::error!("restart_process failed : {:?}", err);
            }
        });
    } else {
        log::info!("configuration updated , restart the program to apply");
    }

    return Ok(());
}

pub fn get_configuration_proto() -> MainboardConfig {
    let mut config = MainboardConfig::new();

    let mut mqtt = MQTTConfig::new();
    mqtt.port = CONFIG.mqtt.port as i32;
    mqtt.url = CONFIG.mqtt.url.clone();

    let mut comboard = ComboardConfig::new();
    comboard.config = CONFIG.comboard.config.clone();
    comboard.imple = CONFIG.comboard.imple.clone();

    let mut comboards = RepeatedField::new();
    for comboard in CONFIG.comboards.iter() {
        comboards.push(ComboardConfig {
            config: comboard.config.clone(),
            imple: comboard.imple.clone(),
            ..Default::default()
        });
    }

    let mut logger = ProtoLoggerConfig::new();
    logger.target = CONFIG.logger.target.clone();

    let mut http = HttpServerConfig::new();
    http.addr = CONFIG.server.addr.clone();
    http.port = CONFIG.server.port as i32;

    let mut update = UpdaterConfig::new();
    update.autoupdate = CONFIG.update.autoupdate;
    update.channel = CONFIG.update.channel.clone();
    update.reboot = CONFIG.update.reboot;

    let mut api = ApiConfig::new();
    api.url = CONFIG.api.url.clone();

    let mut proxy = crate::protos::board::ReverseProxyConfig::new();
    proxy.url = CONFIG.proxy.url.clone();

    config.set_id(CONFIG.id.clone());
    config.set_mqtt(mqtt);
    config.set_logger(logger);
    config.set_server(http);
    config.set_comboard(comboard);
    config.set_update(update);
    config.set_comboards(comboards);
    config.set_api(api);
    config.set_proxy(proxy);

    return config;
}

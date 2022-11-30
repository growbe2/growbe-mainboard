use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::socket::http::get_api_url;
use crate::socket::http::get_token;
use growbe_shared::version::VERSION;

use super::config::CONFIG;
use super::error::MainboardError;

fn get_default_reboot() -> bool {
    false
}

#[derive(Serialize, Deserialize, Default)]
pub struct MainboardUpdateMessage {
    pub id: i32,
    pub version: String,
    pub release: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UpdateConfig {
    pub autoupdate: bool,
    pub channel: String,
    #[serde(default = "get_default_reboot")]
    pub reboot: bool,
}

pub fn get_default_update_config() -> UpdateConfig {
    return UpdateConfig {
        autoupdate: false,
        channel: "".to_string(),
        reboot: true,
    };
}

// download the wanted version
pub fn download_version(version: &String) -> Result<(), MainboardError> {
    let asset_name = "growbe-mainboard-arm-linux";

    let version = if version.contains("-") {
        "latest"
    } else {
        version.as_str()
    };

    let output = Command::new("bash")
        .current_dir("/opt/growbe/")
        .arg("/opt/growbe/download.sh")
        .arg(version)
        .arg(asset_name)
        .output()
        .map_err(|x| MainboardError::from_error(x.to_string()))?;

    std::io::stdout()
        .write_all(&output.stdout)
        .map_err(|x| MainboardError::from_error(x.to_string()))?;
    std::io::stderr()
        .write_all(&output.stderr)
        .map_err(|x| MainboardError::from_error(x.to_string()))?;

    return Ok(());
}

pub fn replace_version(_version: &String) -> Result<(), MainboardError> {
    let asset_name = "growbe-mainboard-arm-linux";

    Command::new("mv")
        .current_dir("/opt/growbe/")
        .arg(asset_name)
        .arg("growbe-mainboard")
        .output()
        .map_err(|x| MainboardError::from_error(x.to_string()))?;

    log::info!("update complete , restart the process to take effect");

    Ok(())
}

// TODO: helllo i'm soom bad code
pub fn get_latest_version() -> Option<String> {
    let (tx, rx) = channel();

    tokio::task::spawn(async move {
        let client = reqwest::Client::new();
        let body_result = client
            .get(get_api_url("/growbe-mainboard/version".to_string()))
            .query(&[("channel", CONFIG.update.channel.as_str())])
            .bearer_auth(get_token())
            .send()
            .await;
        let version = match body_result {
            Ok(body) => match body.json::<MainboardUpdateMessage>().await {
                Ok(body) => body.version,
                Err(err) => {
                    log::debug!("{:?}", err);
                    "".to_string()
                }
            },
            Err(err) => {
                log::debug!("{:?}", err);
                "".to_string()
            }
        };

        tx.send(version).unwrap();
    });
    let version = rx.recv().unwrap();
    if version.eq("") {
        return None;
    }
    return Some(version);
}

pub fn update_available() -> Option<String> {
    let version = get_latest_version();
    if let Some(version) = version {
        let my_version = VERSION.to_string();

        return if version.eq(&my_version) {
            None
        } else {
            log::info!("new version available {} replacing {}", version, my_version);
            Some(version)
        };
    }
    return None;
}

pub fn autoupdate() {
    if CONFIG.update.autoupdate {
        handle_version_update_request();
    }
}

pub fn handle_version_update_request() -> Option<crate::protos::board::UpdateExecute> {
    if let Some(version) = update_available() {
        log::info!("update available {}", version);
        let update_config = &crate::mainboardstate::config::CONFIG.update;

        if let Err(err) = crate::mainboardstate::update::download_version(&version) {
            log::error!("{:?}", err);
            return None;
        }
        if let Err(err) = crate::mainboardstate::update::replace_version(&version) {
            log::error!("{:?}", err);
            return None;
        }

        let mut update_execute = crate::protos::board::UpdateExecute::new();
        update_execute.version = version.clone();

        if update_config.reboot == true {
            println!("Gonna reboot soon");
            tokio::task::spawn(async move {
                println!("waiting to restart");
                tokio::time::sleep(Duration::from_millis(1000)).await;
                if let Err(err) = crate::plateform::restart::restart_process() {
                    log::error!("restart_process failed : {:?}", err);
                }
            });
            tokio::task::spawn(async {});
        }

        Some(update_execute);
    }

    return None;
}

pub fn handle_version_update(
    payload: &crate::protos::board::VersionRelease,
) -> Option<crate::protos::board::UpdateExecute> {
    let update_config = &crate::mainboardstate::config::CONFIG.update;

    if update_config.autoupdate == true {
        if update_config.channel == payload.channel {
            log::info!("receive update for channel {:?}", payload);
            if let Err(err) = crate::mainboardstate::update::download_version(&payload.version) {
                log::error!("{:?}", err);
                return None;
            }
            if let Err(err) = crate::mainboardstate::update::replace_version(&payload.version) {
                log::error!("{:?}", err);
                return None;
            }

            let mut update_execute = crate::protos::board::UpdateExecute::new();
            update_execute.version = payload.version.clone();

            if update_config.reboot == true {
                if let Err(err) = crate::plateform::restart::restart_process() {
                    log::error!("{:?}", err);
                    // gonna return executaed since only the restart remaining
                }
            }

            Some(update_execute);
        } else {
            log::debug!("receive update for other channel {:?}", payload);
        }
    } else {
        log::debug!("receive but not subscribe to autoupdate {:?}", payload);
    }
    None
}

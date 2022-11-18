use serde::{Deserialize, Serialize};
use std::process::Command;
use std::io::Write;


fn get_default_reboot() -> bool {
    false
}

#[derive(Serialize, Deserialize, Default)]
pub struct UpdateConfig {
    pub autoupdate: bool,
    pub channel: String,
	#[serde(default = "get_default_reboot")]
    pub reboot: bool,
}

pub fn get_default_update_config() -> UpdateConfig {
    return UpdateConfig{
        autoupdate: false,
        channel: "".to_string(),
        reboot: true,
    };
}

// download the wanted version 
pub fn download_version(pat: &String, version: &String) -> () {
    let asset_name = "growbe-mainboard-arm-linux";

    let output = Command::new("bash")
        .current_dir("/opt/growbe/")
        .arg("/opt/growbe/download.sh")
        .env("GITHUB_ACCESS_TOKEN", pat)
        .arg(version.as_str())
        .arg(asset_name)
        .arg(format!("{}-{}", asset_name, version.as_str()))
        .output().unwrap();

    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
}

pub fn replace_version(version: &String) -> () {
    let asset_name = "growbe-mainboard-arm-linux";

    Command::new("mv")
        .current_dir("/opt/growbe/")
        .arg(format!("{}-{}", asset_name, version.as_str()))
        .arg("growbe-mainboard")
        .output().unwrap();
}


pub fn handle_version_update(payload: &crate::protos::board::VersionRelease) -> Option<crate::protos::board::UpdateExecute> {
    let update_config = &crate::mainboardstate::config::CONFIG.update;

    if update_config.autoupdate == true {
        if update_config.channel == payload.channel {
            log::info!("receive update for channel {:?}", payload);
            crate::mainboardstate::update::download_version(&"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".to_string(), &payload.version);
            crate::mainboardstate::update::replace_version(&payload.version);

            let mut update_execute = crate::protos::board::UpdateExecute::new();
            update_execute.version = payload.version.clone();

            if update_config.reboot == true {
                crate::plateform::restart::restart_process();
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

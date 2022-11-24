


// restart the service
pub fn restart_process() -> () {
    log::info!("restart process");
    std::process::Command::new("systemctl")
        .arg("restart")
        .arg("growbe-mainboard@dev.service")
        .output().unwrap();
}

pub fn restart_host() -> () {
    std::process::Command::new("reboot")
        .output().unwrap();
}
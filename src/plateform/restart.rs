


// restart the service
pub fn restart() -> () {
    std::process::Command::new("systemctl")
        .arg("restart")
        .arg("growbe-mainboard@dev.service")
        .output().unwrap();
}
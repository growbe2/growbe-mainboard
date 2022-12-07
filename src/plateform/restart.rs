use crate::mainboardstate::error::MainboardError;

// restart the service
pub fn restart_process() -> Result<(), MainboardError> {
    log::info!("restart process");
    std::process::Command::new("systemctl")
        .arg("restart")
        .arg("growbe-mainboard@dev.service")
        .output()
        .map_err(|x| MainboardError::from_error(x.to_string()))?;
    Ok(())
}

pub fn restart_host() -> Result<(), MainboardError> {
    std::process::Command::new("reboot")
        .output()
        .map_err(|x| MainboardError::from_error(x.to_string()))?;
    Ok(())
}

use crate::{protos::board::HostInformation, mainboardstate::error::MainboardError};

pub fn get_host_information() -> Result<HostInformation, MainboardError> {
    let un = uname::uname()
        .map_err(|x| MainboardError::from_error(x.to_string()))?;
    let mut host = HostInformation::new();

    host.architecture = un.machine;
    host.deviceType = un.nodename;
    host.os = un.sysname;
    host.kernelVersion = un.version;
    host.kernel = un.release;

    return Ok(host);
}

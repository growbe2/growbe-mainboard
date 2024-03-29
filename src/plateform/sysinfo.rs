use serde::{Deserialize, Serialize};

use crate::mainboardstate::error::MainboardError;

#[derive(Serialize, Deserialize)]
pub struct SysInfo {
    pub hostname: String,

    pub uname: String,

    pub os: String,

    pub uptime: u64,

    pub load_average: [f64; 3],

    pub ram_total: u64,

    pub ram_unused: u64,
}

#[allow(dead_code)]
pub fn get_sys_info() -> Result<SysInfo, MainboardError> {
    let mut buf = [0u8; 64];
    let hostname = nix::unistd::gethostname(&mut buf)
        .map_err(|x| MainboardError::from_error(x.to_string()))?;

    let uts_name = nix::sys::utsname::uname()?;
    let mut info = SysInfo {
        hostname: hostname.to_str().unwrap_or_default().to_owned(),
        uname: format!("{:?} {:?}", uts_name.machine(), uts_name.release()),
        os: uts_name.sysname().to_os_string().into_string()?,
        uptime: 0,
        load_average: [0., 0., 0.],
        ram_total: 0,
        ram_unused: 0,
    };

    #[cfg(target_os = "linux")]
    {
        let sys_info =
            nix::sys::sysinfo::sysinfo().map_err(|x| MainboardError::from_error(x.to_string()))?;
        let load_average = sys_info.load_average();
        info.uptime = sys_info.uptime().as_secs();
        info.load_average = [load_average.0, load_average.1, load_average.2];
        info.ram_total = sys_info.ram_total();
        info.ram_unused = sys_info.ram_unused();
    }

    return Ok(info);
}

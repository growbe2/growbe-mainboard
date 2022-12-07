use serde::{Deserialize, Serialize};

use nix::sys::socket::AddressFamily;

use crate::mainboardstate::error::MainboardError;

#[derive(Serialize, Deserialize, Clone)]
pub struct NetworkIterface {
    pub name: String,
    pub ip: String,
    pub mac: String,
    pub mask: String,
    pub broadcast: String,
    pub destination: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkInfo {
    interfaces: Vec<NetworkIterface>,
}

pub fn get_ip_addr() -> Result<String, MainboardError> {
    let net_info = get_net_info()?;
    // TODO rework this to work on none pi device using wifi
    let interface_wlan0_response = net_info.interfaces.iter().find(|&x| x.name == "wlan0");
    if let Some(x) = interface_wlan0_response {
        return Ok(x.ip.clone());
    }
    return Err(MainboardError::from_error(
        "failed to get wlan0".to_string(),
    ));
}

pub fn get_net_info() -> Result<NetworkInfo, MainboardError> {
    let addrs =
        nix::ifaddrs::getifaddrs().map_err(|err| MainboardError::from_error(err.to_string()))?;

    let mut hashmap: std::collections::HashMap<String, NetworkIterface> =
        std::collections::HashMap::new();

    for ifaddr in addrs {
        if !hashmap.contains_key(&ifaddr.interface_name) {
            hashmap.insert(
                ifaddr.interface_name.clone(),
                NetworkIterface {
                    ip: String::default(),
                    name: ifaddr.interface_name.clone(),
                    mac: String::default(),
                    broadcast: String::default(),
                    destination: String::default(),
                    mask: String::default(),
                },
            );
        }
        let mut item = hashmap.get_mut(&ifaddr.interface_name).unwrap();
        match ifaddr.address {
            Some(address) => match address.family() {
                AddressFamily::Inet => {
                    item.ip = address.to_string().replace(":0", "");
                    if let Some(netmask) = ifaddr.netmask {
                        item.mask = netmask.to_string();
                    }
                    if let Some(broadcast) = ifaddr.broadcast {
                        item.broadcast = broadcast.to_string().replace(":0", "");
                    }
                    if let Some(destination) = ifaddr.destination {
                        item.destination = destination.to_string();
                    }
                }
                #[cfg(target_os = "linux")]
                AddressFamily::Packet => item.mac = address.to_string(),
                _ => {}
            },
            None => {}
        }
    }

    if hashmap.len() == 0 {
        return Err(MainboardError::new().message("failed to get network interface".to_string()));
    }

    return Ok(NetworkInfo {
        interfaces: hashmap.into_values().collect(),
    });
}

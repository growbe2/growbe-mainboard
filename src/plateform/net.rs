use serde::{Deserialize, Serialize};

use nix::sys::socket::AddressFamily;


#[derive(Serialize, Deserialize)]
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

pub fn get_net_info() -> NetworkInfo {
    let addrs = nix::ifaddrs::getifaddrs().unwrap();

    let mut hashmap: std::collections::HashMap<String, NetworkIterface> = std::collections::HashMap::new();

    for ifaddr in addrs {
        if !hashmap.contains_key(&ifaddr.interface_name) {
            hashmap.insert(ifaddr.interface_name.clone(), NetworkIterface{
                ip: String::default(),
                name: ifaddr.interface_name.clone(),
                mac: String::default(),
                broadcast: String::default(),
                destination: String::default(),
                mask: String::default(),
            });
        }
        let mut item = hashmap.get_mut(&ifaddr.interface_name).unwrap();
        match ifaddr.address {
          Some(address) => {
              match address.family() {
                  AddressFamily::Inet => {
                      item.ip = address.to_string();
                      if let Some(netmask) = ifaddr.netmask {
                          item.mask = netmask.to_string();
                      }
                      if let Some(broadcast) = ifaddr.broadcast {
                          item.broadcast = broadcast.to_string();
                      }
                      if let Some(destination) = ifaddr.destination {
                          item.destination = destination.to_string();
                      }
                  },
                  AddressFamily::Packet => item.mac = address.to_string(),
                  _ => {}
              }
          },
          None => {}
        }
      }

    return NetworkInfo{
        interfaces: hashmap.into_values().collect(),
    }
}
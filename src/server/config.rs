
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HttpServerConfig {
    pub addr: String,
    pub port: u16,
}


pub fn get_default_server_config() -> HttpServerConfig {
    return HttpServerConfig{
        addr: String::from("0.0.0.0"),
        port: 3030
    }
}

use serde::{Deserialize, Serialize};

pub const MSG_READ_MODULE_ID: &str = "READ_MODULE_ID";
pub const MSG_READ_SUPPORTED_MODULES: &str = "READ_SUPPORTED_MODULES";

#[derive(Serialize, Deserialize)]
pub struct WSServerConfig {
    pub addr: String,
    pub port: u16,
}


#[derive(Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub topic: String,
    pub payload: String,
}

impl ToString for WSServerConfig {
	fn to_string(&self) -> String {
		format!("{}:{}", self.addr, self.port)
	}	
}


pub fn default_server_config() -> WSServerConfig {
    return WSServerConfig{
        addr: String::from("0.0.0.0"),
        port: 5000
    }
}

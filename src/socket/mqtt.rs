use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CloudMQTTConfig {
    pub url: String,
    pub port: u16,
}

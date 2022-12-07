use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct CloudMQTTConfig {
    pub url: String,
    pub port: u16,
}

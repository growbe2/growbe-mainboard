

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct CloudMQTTConfig {
	pub url: String,
	pub port: u16,
}


use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ComboardConfig {
	pub imple: String,
	pub config: String,
}
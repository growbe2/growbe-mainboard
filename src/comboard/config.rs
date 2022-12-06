use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct ComboardConfig {
    pub imple: String,
    pub config: String,
}

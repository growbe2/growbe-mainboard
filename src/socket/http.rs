use serde::{Deserialize, Serialize,};

use crate::mainboardstate::config::CONFIG;

pub fn get_default_api_config() -> APIConfig {
	return APIConfig{
        url: "https://api.growbe.ca".to_string()
	}
}

#[derive(Serialize, Deserialize, Default)]
pub struct APIConfig {
	pub url: String,
}


pub fn get_api_url(path: String) -> String {
    return format!("{}{}", CONFIG.api.url, path);
}

pub fn get_token() -> String {
    return if let Ok(token) = std::env::var("TOKEN") { token } else { "".to_string() };
}

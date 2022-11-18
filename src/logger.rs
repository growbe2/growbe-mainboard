use serde::{Deserialize, Serialize,};

pub fn default_logger() -> LoggerConfig {
	return LoggerConfig{
		target: String::from("growbe_mainboard=warn"),
		systemd: false,
	}
}

#[derive(Serialize, Deserialize, Default)]
pub struct LoggerConfig {
	pub target: String,
    #[serde(default)] 
	pub systemd: bool,
}


pub fn setup_log() {
	env_logger::Builder::from_env(
		env_logger::Env::default().default_filter_or(crate::mainboardstate::config::CONFIG.logger.target.as_str())
	).init();
}
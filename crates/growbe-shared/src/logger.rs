use serde::{Deserialize, Serialize,};

use std::io::Write;



#[derive(Serialize, Deserialize, Default)]
pub struct LoggerConfig {
	pub target: String,
    #[serde(default)] 
	pub systemd: bool,
}


pub fn setup_log(config: &LoggerConfig) {
	env_logger::Builder::from_env(
	env_logger::Env::default().default_filter_or(config.target.as_str())
	)
	.format(|buf, record| writeln!(buf, "[{} {}]: {}", record.target(), record.level(), record.args()))
	.init();
}
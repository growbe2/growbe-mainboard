
use serde::{Deserialize, Serialize};

use crate::logger::{LoggerConfig, default_logger};
use crate::modules::Module;
use crate::ws::{WSServerConfig, default_server_config};

fn get_config_path() -> String {
	let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 ||  args[1].is_empty() {
        panic!("config not passed as args[1]");
    }
	return args[1].clone()
}

lazy_static::lazy_static! {
	pub static ref CONFIG: MainboardProcessConfig = {
	    return get(&get_config_path()).unwrap();
	};
}

#[derive(Serialize, Deserialize)]
pub struct MainboardProcessConfig {
    #[serde(default)] 
	pub id: String,
	#[serde(default = "default_logger")]
	pub logger: LoggerConfig,
	#[serde(default = "default_server_config")]
	pub server: WSServerConfig,
	
	pub modules: Vec<Module>,
}


pub fn get(config: &String) -> Result<MainboardProcessConfig, serde_json::Error>  {
    let file = std::fs::File::open(config).expect("Error open file");
    let scenario: MainboardProcessConfig = serde_json::from_reader(file)?;
    Ok(scenario)
}
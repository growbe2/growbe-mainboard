
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
	pub static ref CONFIG: MainboardProcessConfig = {
		let args: Vec<String> = std::env::args().collect();
	    if args.len() == 1 ||  args[1].is_empty() {
	        panic!("config not passed as args[1]");
	    }
	    return get(&args[1]).unwrap();
	};
}

#[derive(Serialize, Deserialize)]
pub struct MainboardProcessConfig {
    #[serde(default)] 
	pub id: String,
	pub mqtt: crate::socket::mqtt::CloudMQTTConfig,
}


pub fn get(config: &String) -> Result<MainboardProcessConfig, serde_json::Error>  {
    let file = std::fs::File::open(config).expect("Error open file");
    let scenario: MainboardProcessConfig = serde_json::from_reader(file)?;
    Ok(scenario)
}


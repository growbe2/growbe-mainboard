use crate::protos::board::RunningComboard;

use self::{imple::{interface::ComboardClientConfig}};

#[cfg(feature = "com_ble")]
use self::imple::ble::get_ble_comboard;
#[cfg(feature = "com_ws")]
use self::imple::ws::get_ws_comboard;


pub mod imple;
pub mod config;

#[cfg(all(target_os = "linux", feature = "com_i2c"))]
fn get_comboard_i2c(config: ComboardClientConfig) -> imple::i2c_linux::I2CLinuxComboardClient {
	return imple::i2c_linux::I2CLinuxComboardClient {
		config_comboard: config
	};
}

#[cfg(feature = "com_virt")]
fn get_comboard_virt(config: ComboardClientConfig) -> imple::virt::VirtualComboardClient {
	return imple::virt::VirtualComboardClient {
		config_comboard: config
	};
}

pub fn get_comboard_client() -> (Vec<Box<dyn imple::interface::ComboardClient>>, Vec<RunningComboard>) {
	let mut boards: Vec<Box<dyn imple::interface::ComboardClient>>  = vec![];
	let mut running_boards: Vec<RunningComboard> = vec![];

	let imple = crate::mainboardstate::config::CONFIG.comboard.imple.as_str();
	if imple == "i2c" {
		#[cfg(all(target_os = "linux", feature = "com_i2c"))]
		boards.push(Box::new(get_comboard_i2c(ComboardClientConfig { config: crate::mainboardstate::config::CONFIG.comboard.config.to_string() })));
		#[cfg(not(all(target_os = "linux", feature = "com_i2c")))]
		panic!("i2c not supported on this os")
	}

	for element in crate::mainboardstate::config::CONFIG.comboards.iter() {
		if element.imple == "virt" {
			#[cfg(feature = "com_virt")]
			boards.push(Box::new(get_comboard_virt(ComboardClientConfig { config: element.config.clone() })));
			#[cfg(not(feature = "com_virt"))]
			panic!("virt not build")
		} else if element.imple == "i2c" {
			#[cfg(all(target_os = "linux", feature = "com_i2c"))]
			boards.push(Box::new(get_comboard_i2c(ComboardClientConfig { config: element.config.clone() })));
			#[cfg(not(all(target_os = "linux", feature = "com_i2c")))]
			panic!("i2c not supported on this os")
		} else if element.imple == "ble" {
			#[cfg(feature = "com_ble")]
			boards.push(get_ble_comboard(element.config.clone()));
			#[cfg(not(feature = "com_ble"))]
			panic!("ble not build")
		} else if element.imple == "ws" {
			#[cfg(feature = "com_ws")]
			boards.push(get_ws_comboard(element.config.clone()));
			#[cfg(not(feature = "com_ws"))]
			panic!("ws not build")
		}

		running_boards.push(RunningComboard { imple: element.imple.clone(), addr: element.config.clone(), ..Default::default()})
	}

	return (boards, running_boards);
}
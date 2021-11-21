pub mod imple;
pub mod config;

fn get_comboard_i2c() -> imple::i2c_linux::I2CLinuxComboardClient {
	return imple::i2c_linux::I2CLinuxComboardClient {};
}

fn get_comboard_virt() -> imple::virt::VirtualComboardClient {
	return imple::virt::VirtualComboardClient {};
}

pub fn get_comboard_client() -> Box<dyn imple::interface::ComboardClient>  {
	let imple = crate::mainboardstate::config::CONFIG.comboard.imple.as_str();
	if imple == "i2c" {
		return Box::new(get_comboard_i2c());
	}
	return Box::new(get_comboard_virt());
}
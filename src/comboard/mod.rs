pub mod imple;
pub mod config;

#[cfg(target_arch = "arm")]
fn get_comboard_for_plateform() -> imple::i2c_linux::I2CLinuxComboardClient {
	return imple::i2c_linux::I2CLinuxComboardClient {};
}

#[cfg(target_arch = "x86_64")]
fn get_comboard_for_plateform() -> imple::virt::VirtualComboardClient {
	return imple::virt::VirtualComboardClient {};
}

pub fn get_comboard_client() -> Box<dyn imple::interface::ComboardClient>  {
	return std::boxed::Box::new(get_comboard_for_plateform());
}
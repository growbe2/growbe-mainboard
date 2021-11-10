pub mod imple;

#[cfg(target_arch = "arm")]
fn getComboardForPlateform() -> imple::i2c_linux::I2CLinuxComboardClient {
	return imple::i2c_linux::I2CLinuxComboardClient {};
}

#[cfg(target_arch = "x86_64")]
fn getComboardForPlateform() -> imple::virt::VirtualComboardClient {
	return imple::virt::VirtualComboardClient {};
}

pub fn getComboardClient() -> Box<dyn imple::interface::ComboardClient>  {
	return std::boxed::Box::new(getComboardForPlateform());
}
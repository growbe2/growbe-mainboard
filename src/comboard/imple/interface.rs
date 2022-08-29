
pub static I2C_BOARD_ID: &'static str = "i2c";
pub static I2C_VIRT_ID: &'static str = "virt";
pub static I2C_BLE_ID: &'static str = "ble";

#[repr(C)]
pub struct Module_Config {
    pub port: cty::c_int,
    pub buffer: [cty::uint8_t; 8],
}

pub struct ModuleStateChangeEvent {
    pub board: String,
    pub board_addr: String,
    pub port: i32,
    pub id: String,
    pub state: bool,
}

pub struct ModuleValueValidationEvent {
    pub board: String,
    pub board_addr: String,
    pub port: i32,
    pub buffer: Vec<u8>,
}

pub struct ComboardClientConfig {
    pub config: String,
}

pub trait ComboardClient {
	fn run(&self) -> tokio::task::JoinHandle<()>;
}

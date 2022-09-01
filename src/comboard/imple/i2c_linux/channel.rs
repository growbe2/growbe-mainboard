use std::sync::{Mutex};
use std::sync::mpsc::{Sender, Receiver};

use cty::{c_int, uint8_t};

use crate::comboard::imple::channel::mutex_channel;


#[repr(C)]
pub struct Module_Config {
    pub port: c_int,
    pub buffer: [uint8_t; 8],
}

lazy_static::lazy_static! {
	pub static ref CHANNEL_CONFIG_I2C:(Mutex<Sender<Module_Config>>, Mutex<Receiver<Module_Config>>) = mutex_channel();
}


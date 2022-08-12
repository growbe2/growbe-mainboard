
use std::sync::{Mutex, mpsc::channel,};
use std::sync::mpsc::{Sender, Receiver};

use super::interface::{ModuleValueValidationEvent, ModuleStateChangeEvent};

fn mutex_channel<T>() -> (Mutex<Sender<T>>, Mutex<Receiver<T>>) {
	let (sender, receive) = channel::<T>();

	return (Mutex::new(sender), Mutex::new(receive));
}

lazy_static::lazy_static! {
	pub static ref CHANNEL_CONFIG_I2C:(Mutex<Sender<super::interface::Module_Config>>, Mutex<Receiver<super::interface::Module_Config>>) = mutex_channel();
	pub static ref CHANNEL_CONFIG_VIRT:(Mutex<Sender<super::interface::Module_Config>>, Mutex<Receiver<super::interface::Module_Config>>) = mutex_channel();
	pub static ref CHANNEL_CONFIG_BLE:(Mutex<Sender<super::interface::Module_Config>>, Mutex<Receiver<super::interface::Module_Config>>) = mutex_channel();
	pub static ref CHANNEL_CONFIG_SOCK:(Mutex<Sender<super::interface::Module_Config>>, Mutex<Receiver<super::interface::Module_Config>>) = mutex_channel();

	pub static ref CHANNEL_STATE:(Mutex<Sender<super::interface::ModuleStateChangeEvent>>, Mutex<Receiver<super::interface::ModuleStateChangeEvent>>)  = mutex_channel();
	pub static ref CHANNEL_VALUE:(Mutex<Sender<super::interface::ModuleValueValidationEvent>>, Mutex<Receiver<super::interface::ModuleValueValidationEvent>>)  = mutex_channel();
}



pub fn comboard_send_value(board: String, board_addr: String, port: i32, buffer: Vec<u8>) -> Result<(), ()> {
	return CHANNEL_VALUE.0.lock().unwrap().send(
        ModuleValueValidationEvent{
            port: port,
            board: board,
            board_addr: board_addr,
            buffer: buffer,
        }
    ).map_err(|_| ());
}

pub fn comboard_send_state(board: String, board_addr: String, port: i32, id: String, state: bool) -> Result<(), ()> {
	return CHANNEL_STATE.0.lock().unwrap().send(
		ModuleStateChangeEvent{
			board: board,
			board_addr: board_addr,
			port: port,
			id: id,
			state: state,
		}
	).map_err(|_| ());
}

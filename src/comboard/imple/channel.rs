
use std::sync::{Mutex, mpsc::channel,};
use std::sync::mpsc::{Sender, Receiver};

fn mutex_channel<T>() -> (Mutex<Sender<T>>, Mutex<Receiver<T>>) {
	let (sender, receive) = channel::<T>();

	return (Mutex::new(sender), Mutex::new(receive));
}

lazy_static::lazy_static! {
	pub static ref CHANNEL_CONFIG:(Mutex<Sender<super::interface::Module_Config>>, Mutex<Receiver<super::interface::Module_Config>>) = mutex_channel();
	pub static ref CHANNEL_STATE:(Mutex<Sender<super::interface::ModuleStateChangeEvent>>, Mutex<Receiver<super::interface::ModuleStateChangeEvent>>)  = mutex_channel();
	pub static ref CHANNEL_VALUE:(Mutex<Sender<super::interface::ModuleValueValidationEvent>>, Mutex<Receiver<super::interface::ModuleValueValidationEvent>>)  = mutex_channel();
}

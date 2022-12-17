use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::sync::Mutex;

use cty::{c_int, uint8_t};

use super::super::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};

#[repr(C)]
#[derive(Debug)]
pub struct Module_Config {
    pub port: c_int,
    pub buffer: [uint8_t; 8],
}

pub fn mutex_channel_tokio<T>() -> (Mutex<tokio::sync::mpsc::Sender<T>>, Mutex<tokio::sync::mpsc::Receiver<T>>) {
    let (sender, receive) = channel::<T>(10);

    return (Mutex::new(sender), Mutex::new(receive));
}

lazy_static::lazy_static! {
    pub static ref CHANNEL_CONFIG_I2C:(Mutex<Sender<Module_Config>>, Mutex<Receiver<Module_Config>>) = mutex_channel_tokio();
    pub static ref CHANNEL_STATE:(Mutex<Sender<ModuleStateChangeEvent>>, Mutex<Receiver<ModuleStateChangeEvent>>)  = mutex_channel_tokio();
    pub static ref CHANNEL_VALUE:(Mutex<Sender<ModuleValueValidationEvent>>, Mutex<Receiver<ModuleValueValidationEvent>>)  = mutex_channel_tokio();
}

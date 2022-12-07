use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::sync::{mpsc::channel, Mutex};

use crate::mainboardstate::error::MainboardError;

use super::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};

pub fn mutex_channel<T>() -> (Mutex<Sender<T>>, Mutex<Receiver<T>>) {
    let (sender, receive) = channel::<T>();

    return (Mutex::new(sender), Mutex::new(receive));
}

lazy_static::lazy_static! {
    pub static ref CHANNEL_STATE:(Mutex<Sender<super::interface::ModuleStateChangeEvent>>, Mutex<Receiver<super::interface::ModuleStateChangeEvent>>)  = mutex_channel();
    pub static ref CHANNEL_VALUE:(Mutex<Sender<super::interface::ModuleValueValidationEvent>>, Mutex<Receiver<super::interface::ModuleValueValidationEvent>>)  = mutex_channel();
}

pub fn comboard_send_value(
    board: String,
    board_addr: String,
    port: i32,
    buffer: Vec<u8>,
) -> Result<(), ()> {
    return CHANNEL_VALUE
        .0
        .lock()
        .unwrap()
        .send(ModuleValueValidationEvent {
            port: port,
            board: board,
            board_addr: board_addr,
            buffer: buffer,
        })
        .map_err(|_| ());
}

pub fn comboard_send_state(
    board: String,
    board_addr: String,
    port: i32,
    id: String,
    state: bool,
) -> Result<(), ()> {
    return CHANNEL_STATE
        .0
        .lock()
        .unwrap()
        .send(ModuleStateChangeEvent {
            board: board,
            board_addr: board_addr,
            port: port,
            id: id,
            state: state,
        })
        .map_err(|_| ());
}

pub trait ModuleDataContainer<T> {
    fn get_data(&self) -> Result<T, ()>;
}

pub struct ComboardAddr {
    pub imple: String,
    pub addr: String,
}

impl ToString for ComboardAddr {
    fn to_string(&self) -> String {
        return format!("{}:{}", self.imple, self.addr);
    }
}

pub struct ModuleConfig {
    pub port: i32,
    pub data: Vec<u8>,
}

pub type ComboardSenderMap = HashMap<String, Sender<ModuleConfig>>;

pub struct ComboardSenderMapReference {
    map: Arc<Mutex<ComboardSenderMap>>,
}

impl ComboardSenderMapReference {
    /*
    pub fn send(&self, module_addr: ComboardAddr, module_config: ModuleConfig) -> Result<(), ()> {
        let addr = module_addr.to_string();

        if let Some(sender) = &self.map.lock().unwrap().get(&addr) {
            return sender.send(module_config).map_err(|_| ());
        }

        return Err(());
    }
    */

    pub fn get_sender(
        &self,
        module_addr: ComboardAddr,
    ) -> Result<Sender<ModuleConfig>, MainboardError> {
        let addr = module_addr.to_string();

        if let Some(sender) = &self.map.lock().unwrap().get(&addr) {
            return Ok((*sender).clone());
        }

        return Err(MainboardError::not_found("comboard_sender", addr.as_str()));
    }
}

pub struct ComboardConfigChannelManager {
    senders: Arc<Mutex<ComboardSenderMap>>,
}

impl ComboardConfigChannelManager {
    pub fn new() -> Self {
        return ComboardConfigChannelManager {
            senders: Arc::new(Mutex::new(ComboardSenderMap::new())),
        };
    }

    pub fn create_channel(&mut self, addr: ComboardAddr) -> Receiver<ModuleConfig> {
        let (sender, receiver) = channel::<ModuleConfig>();
        self.senders
            .lock()
            .unwrap()
            .insert(addr.to_string(), sender);
        log::info!("creating channel for {:?}", addr.to_string());
        return receiver;
    }

    pub fn get_reference(&self) -> ComboardSenderMapReference {
        return ComboardSenderMapReference {
            map: self.senders.clone(),
        };
    }
}

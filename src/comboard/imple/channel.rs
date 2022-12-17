use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::mainboardstate::error::MainboardError;

use super::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};

/*
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
        .try_send(ModuleValueValidationEvent {
            port,
            board,
            board_addr,
            buffer,
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
        .try_send(ModuleStateChangeEvent {
            board,
            board_addr,
            port,
            id,
            state,
        })
        .map_err(|_| ());
}
*/

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
        let (sender, receiver) = channel::<ModuleConfig>(50);
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

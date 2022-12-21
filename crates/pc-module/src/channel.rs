use std::collections::HashMap;

use tokio::sync::mpsc::{channel, Receiver, Sender};

pub type ModuleValue = (String, Vec<u8>);


pub struct ModuleConfig {
    pub port: i32,
    pub data: Vec<u8>,
}

pub type ModuleSenderMap = HashMap<String, Sender<ModuleConfig>>;

pub struct ModuleSenderMapReference {
    map: ModuleSenderMap,
}

impl ModuleSenderMapReference {
    pub fn send(&self, port: i32, module_config: ModuleConfig) -> Result<(), ()> {
        let addr = port.to_string();

        if let Some(sender) = &self.map.lock().unwrap().get(&addr) {
            return sender.send(module_config).map_err(|_| ());
        }

        return Err(());
    }

    pub fn get_sender(&self, module_addr: i32) -> Result<Sender<ModuleConfig>, ()> {
        let addr = module_addr.to_string();

        if let Some(sender) = &self.map.lock().unwrap().get(&addr) {
            return Ok((*sender).clone());
        }

        log::error!("failed to get sender {}", addr);
        return Err(());
    }
}

pub struct ModuleConfigChannelManager {
    senders: Arc<Mutex<ModuleSenderMap>>,
}

impl ModuleConfigChannelManager {
    pub fn new() -> Self {
        return ModuleConfigChannelManager {
            senders: Arc::new(Mutex::new(ModuleSenderMap::new())),
        };
    }

    pub fn create_channel(&mut self, addr: i32) -> Receiver<ModuleConfig> {
        let (sender, receiver) = channel::<ModuleConfig>(5);
        self.senders
            .lock()
            .unwrap()
            .insert(addr.to_string(), sender);
        log::info!("creating channel for {:?}", addr.to_string());
        return receiver;
    }

    pub fn get_reference(&self) -> ModuleSenderMapReference {
        return ModuleSenderMapReference {
            map: self.senders.clone(),
        };
    }
}
